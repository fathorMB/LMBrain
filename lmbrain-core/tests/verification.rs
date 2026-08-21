use std::{collections::BTreeMap, ffi::OsString, fs, path::Path};
use tempfile::tempdir;
use lmbrain_core::{
    frontmatter::Document,
    verification::{
        approve_verification_manifest, canonical_verification_manifest_digest,
        execute_spec_verification,
        manifest::{hex_digest, MAX_FINGERPRINT_EXCLUDES, MAX_FINGERPRINT_EXCLUDE_BYTES},
        minimal_gate_environment_from, render_transcript, replace_transcript,
        section_at_level, transcript_state, transcript_state_for_document,
        validate_verification_manifest, workspace_content_fingerprint,
        write_verification_transcript, TranscriptState, VerificationError,
        VerificationGate, VerificationGateResult, VerificationManifest,
        GENERATED_TRANSCRIPT_END, GENERATED_TRANSCRIPT_START, VERIFICATION_MANIFEST_PATH,
    },
};

fn manifest(program: &str) -> VerificationManifest {
        VerificationManifest {
            schema_version: 1,
            gates: vec![VerificationGate {
                id: "sample".into(),
                title: None,
                program: program.into(),
                args: Vec::new(),
                cwd: ".".into(),
                timeout_seconds: Some(5),
                output_limit_bytes: Some(4096),
                expected_exit_code: Some(0),
                result_matcher: None,
                environment: BTreeMap::new(),
                fingerprint_exclude: Vec::new(),
            }],
        }
    }

    #[test]
    fn validates_strict_manifest_and_digest() {
        let valid = manifest("rustc");
        assert!(validate_verification_manifest(&valid).is_empty());
        assert_eq!(
            canonical_verification_manifest_digest(&valid)
                .unwrap()
                .len(),
            64
        );
        let mut invalid = valid.clone();
        invalid.gates[0].program = "../bad".into();
        assert!(!validate_verification_manifest(&invalid).is_empty());
    }

    #[test]
    fn windows_minimal_environment_preserves_program_data_only_as_a_system_root() {
        let inherited = [
            (OsString::from("PATH"), OsString::from(r"C:\\Tools")),
            (
                OsString::from("ProgramData"),
                OsString::from(r"C:\\ProgramData"),
            ),
            (
                OsString::from("SESSION_SECRET"),
                OsString::from("must-not-leak"),
            ),
        ];

        let windows = minimal_gate_environment_from(inherited.clone(), true);
        assert_eq!(
            windows.preserved.get(&OsString::from("ProgramData")),
            Some(&OsString::from(r"C:\\ProgramData"))
        );
        assert!(!windows
            .preserved
            .contains_key(&OsString::from("SESSION_SECRET")));

        let non_windows = minimal_gate_environment_from(inherited, false);
        assert!(!non_windows
            .preserved
            .contains_key(&OsString::from("ProgramData")));
        assert!(!non_windows
            .preserved
            .contains_key(&OsString::from("SESSION_SECRET")));
    }

    #[test]
    fn removed_environment_diagnostics_are_normalized_without_values() {
        let inherited = [
            (OsString::from("Path"), OsString::from("allowed")),
            (OsString::from("Mixed_Case"), OsString::from("sensitive")),
            (OsString::from("mixed_case"), OsString::from("other-secret")),
        ];

        let windows = minimal_gate_environment_from(inherited.clone(), true);
        assert_eq!(windows.removed, vec!["MIXED_CASE"]);
        assert!(!format!("{:?}", windows.removed).contains("sensitive"));

        let non_windows = minimal_gate_environment_from(inherited, false);
        assert_eq!(non_windows.removed, vec!["Mixed_Case", "mixed_case"]);

        let transcript = render_transcript(
            "manifest",
            "before",
            "after",
            "contract",
            &[VerificationGateResult {
                id: "sample".into(),
                command: "check".into(),
                started_at: "start".into(),
                finished_at: "finish".into(),
                duration_ms: 1,
                exit_code: Some(1),
                timed_out: false,
                expectation_met: false,
                removed_environment_variables: windows.removed,
                stdout: String::new(),
                stderr: String::new(),
            }],
            None,
            None,
        );
        assert!(transcript.contains("removed_environment_variables: MIXED_CASE"));
        assert!(transcript.contains("environment_policy: minimal-inherited-allowlist"));
        assert!(!transcript.contains("sensitive"));
        assert!(!transcript.contains("other-secret"));
    }

    #[test]
    fn transcript_presence_is_scoped_and_fenced() {
        assert_eq!(
            transcript_state(
                Path::new("."),
                "## Other\n### Verification transcript\n```\n```"
            ),
            TranscriptState::Missing
        );
        assert_eq!(
            transcript_state(
                Path::new("."),
                "## Implementation evidence\n### Verification transcript\n```\n```"
            ),
            TranscriptState::Empty
        );
        assert_eq!(
            transcript_state(
                Path::new("."),
                "## Implementation evidence\n### Verification transcript\n```text\n$ true\nok\n```"
            ),
            TranscriptState::HandAuthored
        );
    }

    #[test]
    fn fenced_markdown_headings_are_opaque_transcript_content() {
        // KIT-NOTE-005: pasted report Markdown with `# `/`## ` lines inside
        // the transcript fence must not truncate the section or make a valid
        // transcript look empty.
        let body = "## Implementation evidence\n\n### Verification transcript\n\n```text\n$ dotnet run --report\n# SPEC-018 talent shape\n## Batting distribution\nall checks passed\n```\n\n## Next section\nUnrelated.\n";
        assert_eq!(
            transcript_state(Path::new("."), body),
            TranscriptState::HandAuthored
        );

        // The extracted section keeps the whole fence and stops at the real
        // next heading outside it.
        let implementation = section_at_level(body, "Implementation evidence", 2).unwrap();
        let section = section_at_level(implementation, "Verification transcript", 3).unwrap();
        assert!(section.contains("# SPEC-018 talent shape"));
        assert!(section.contains("all checks passed"));
        assert!(!section.contains("Unrelated"));
    }

    #[test]
    fn replace_transcript_ignores_headings_inside_fences() {
        let body = "## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\n### not a section boundary\n## nor this\nok\n```\n\n### Notes\nKeep me.\n";
        let updated = replace_transcript(body, "generated line").unwrap();
        // The managed region lands inside the transcript section, after the
        // hand-authored fence, and the fenced heading-like lines survive.
        assert!(updated.contains("### not a section boundary"));
        assert!(updated.contains("## nor this"));
        assert!(updated.contains(GENERATED_TRANSCRIPT_START));
        assert!(updated.contains("Keep me."));
        let managed_at = updated.find(GENERATED_TRANSCRIPT_START).unwrap();
        let fenced_heading_at = updated.find("### not a section boundary").unwrap();
        let notes_at = updated.find("### Notes").unwrap();
        assert!(fenced_heading_at < managed_at && managed_at < notes_at);
    }

    #[test]
    fn approval_is_digest_and_workspace_bound() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join(".lmbrain")).unwrap();
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&manifest("rustc")).unwrap(),
        )
        .unwrap();
        let approval_path = dir.path().join("local/approval.json");
        let approval = approve_verification_manifest(dir.path(), &approval_path).unwrap();
        assert_eq!(approval.manifest_digest.len(), 64);
        assert!(approval_path.exists());
    }

    #[test]
    fn workspace_fingerprint_ignores_managed_spec_evidence() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::write(dir.path().join("source.txt"), "one").unwrap();
        let first = workspace_content_fingerprint(dir.path()).unwrap();
        fs::write(
            dir.path().join(".lmbrain/specs/working/SPEC-001.md"),
            "evidence",
        )
        .unwrap();
        assert_eq!(first, workspace_content_fingerprint(dir.path()).unwrap());
        fs::write(dir.path().join("source.txt"), "two").unwrap();
        assert_ne!(first, workspace_content_fingerprint(dir.path()).unwrap());
    }

    #[test]
    fn executor_records_real_success_and_red_results_without_rewriting_them() {
        for (name, args, expected_green) in [
            ("green", vec!["--version".to_string()], true),
            (
                "red",
                vec!["--definitely-invalid-lmbrain-test-option".to_string()],
                false,
            ),
        ] {
            let dir = tempdir().unwrap();
            fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
            let mut configured = manifest("rustc");
            configured.gates[0].args = args;
            fs::write(
                dir.path().join(VERIFICATION_MANIFEST_PATH),
                toml::to_string(&configured).unwrap(),
            )
            .unwrap();
            let spec = dir
                .path()
                .join(format!(".lmbrain/specs/working/SPEC-{name}.md"));
            fs::write(&spec, format!(
                "---\nid: SPEC-{name}\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n"
            )).unwrap();
            let approval = dir.path().join("local/approval.json");
            approve_verification_manifest(dir.path(), &approval).unwrap();
            let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
            assert_eq!(report.all_expectations_met, expected_green);
            let source = fs::read_to_string(&spec).unwrap();
            assert!(source.contains("generated-by: lmbrain-verify"));
            assert!(source.contains("$ manual-check\nmanual result"));
            assert!(source.contains(&format!("expectation_met: {expected_green}")));
            let document = Document::parse(&source).unwrap();
            assert_eq!(
                transcript_state(dir.path(), &document.body),
                TranscriptState::GeneratedFresh
            );
            assert_eq!(
                transcript_state_for_document(dir.path(), &document),
                TranscriptState::GeneratedFresh
            );
            let mut changed_contract = document.clone();
            changed_contract.set("verification_gates", "[other-gate]");
            assert_eq!(
                transcript_state_for_document(dir.path(), &changed_contract),
                TranscriptState::GeneratedStale
            );
            let tampered = document.body.replace(
                &format!("expectation_met: {expected_green}"),
                &format!("expectation_met: {}", !expected_green),
            );
            assert_eq!(
                transcript_state(dir.path(), &tampered),
                TranscriptState::GeneratedStale
            );
        }
    }

    #[test]
    fn workspace_mutation_during_gates_invalidates_the_transcript() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::write(dir.path().join("source.txt"), "original").unwrap();
        let mut configured = manifest(if cfg!(windows) { "cmd" } else { "sh" });
        configured.gates[0].args = if cfg!(windows) {
            vec!["/C".into(), "echo mutated>> source.txt".into()]
        } else {
            vec!["-c".into(), "echo mutated >> source.txt".into()]
        };
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-mut.md");
        fs::write(&spec, "---\nid: SPEC-mut\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
        assert!(report.invalidated);
        assert!(
            !report.all_expectations_met,
            "a run that mutated the workspace must not publish success"
        );
        assert_ne!(
            report.workspace_fingerprint_before,
            report.workspace_fingerprint
        );
        assert!(report
            .invalidation_reason
            .as_deref()
            .unwrap()
            .contains("changed during gate execution"));

        let source = fs::read_to_string(&spec).unwrap();
        assert!(source.contains("workspace-fingerprint-before:"));
        assert!(source.contains("<!-- invalidated: workspace content changed"));
        // The decisive regression: the current workspace now matches the
        // recorded post-gate fingerprint, yet the evidence must stay stale.
        assert_eq!(
            workspace_content_fingerprint(dir.path()).unwrap(),
            report.workspace_fingerprint
        );
        let document = Document::parse(&source).unwrap();
        assert_eq!(
            transcript_state(dir.path(), &document.body),
            TranscriptState::GeneratedStale
        );
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedStale
        );
    }

    fn shell_gate(script_windows: &str, script_unix: &str) -> VerificationManifest {
        let mut configured = manifest(if cfg!(windows) { "cmd" } else { "sh" });
        configured.gates[0].args = if cfg!(windows) {
            vec!["/C".into(), script_windows.into()]
        } else {
            vec!["-c".into(), script_unix.into()]
        };
        configured
    }

    #[test]
    fn declared_artifact_outputs_stay_snapshot_consistent() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::create_dir_all(dir.path().join("apps/client/dist")).unwrap();
        fs::write(dir.path().join("source.txt"), "original").unwrap();
        let mut configured = shell_gate(
            "echo bundle> apps\\client\\dist\\out.js",
            "echo bundle > apps/client/dist/out.js",
        );
        configured.gates[0].fingerprint_exclude = vec!["apps/client/dist".into()];
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-dist.md");
        fs::write(&spec, "---\nid: SPEC-dist\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
        assert!(!report.invalidated, "{:?}", report.invalidation_reason);
        assert!(report.invalidation_reason.is_none());
        assert_eq!(
            report.workspace_fingerprint_before,
            report.workspace_fingerprint
        );
        assert!(report.all_expectations_met);

        let document = Document::parse(&fs::read_to_string(&spec).unwrap()).unwrap();
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedFresh
        );

        // A later rebuild touching only the declared output stays fresh...
        fs::write(dir.path().join("apps/client/dist/out.js"), "rebuilt").unwrap();
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedFresh
        );
        // ...while any non-excluded change is still detected.
        fs::write(dir.path().join("source.txt"), "changed").unwrap();
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedStale
        );
    }

    #[test]
    fn mutation_outside_declared_outputs_still_invalidates_with_a_hint() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::create_dir_all(dir.path().join("apps/client/dist")).unwrap();
        fs::write(dir.path().join("source.txt"), "original").unwrap();
        let mut configured = shell_gate("echo mutated>> source.txt", "echo mutated >> source.txt");
        configured.gates[0].fingerprint_exclude = vec!["apps/client/dist".into()];
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-leak.md");
        fs::write(&spec, "---\nid: SPEC-leak\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
        assert!(report.invalidated);
        assert!(report
            .invalidation_reason
            .as_deref()
            .unwrap()
            .contains("fingerprint_exclude"));
    }

    #[test]
    fn exclusion_free_manifests_keep_their_canonical_digest() {
        let plain = manifest("rustc");
        let encoded = serde_json::to_string(&plain).unwrap();
        assert!(
            !encoded.contains("fingerprint_exclude"),
            "an empty exclusion list must not enter the canonical serialization"
        );
        // A manifest that declares exclusions produces a different digest, so
        // materializing the capability always requires operator re-approval.
        let mut excluding = plain.clone();
        excluding.gates[0].fingerprint_exclude = vec!["dist".into()];
        assert_ne!(
            canonical_verification_manifest_digest(&plain).unwrap(),
            canonical_verification_manifest_digest(&excluding).unwrap()
        );
    }

    #[test]
    fn adding_exclusions_invalidates_an_existing_approval() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        let mut configured = manifest("rustc");
        configured.gates[0].args = vec!["--version".into()];
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-appr.md");
        fs::write(&spec, "---\nid: SPEC-appr\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        configured.gates[0].fingerprint_exclude = vec!["dist".into()];
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            execute_spec_verification(dir.path(), &spec, &approval),
            Err(VerificationError::ApprovalRequired)
        ));
    }

    #[test]
    fn rejects_unsafe_fingerprint_exclusions() {
        for entry in [
            "/abs/dist",
            "../outside",
            ".",
            "./",
            "",
            "   ",
            ".lmbrain",
            ".lmbrain/handoffs",
            ".lmbrain\\handoffs",
        ] {
            let mut candidate = manifest("rustc");
            candidate.gates[0].fingerprint_exclude = vec![entry.into()];
            assert!(
                !validate_verification_manifest(&candidate).is_empty(),
                "expected rejection for {entry:?}"
            );
        }

        let mut oversized = manifest("rustc");
        oversized.gates[0].fingerprint_exclude =
            vec!["x".repeat(MAX_FINGERPRINT_EXCLUDE_BYTES + 1)];
        assert!(!validate_verification_manifest(&oversized).is_empty());

        let mut too_many = manifest("rustc");
        too_many.gates[0].fingerprint_exclude = (0..=MAX_FINGERPRINT_EXCLUDES)
            .map(|index| format!("dist-{index}"))
            .collect();
        assert!(!validate_verification_manifest(&too_many).is_empty());

        let mut valid = manifest("rustc");
        valid.gates[0].fingerprint_exclude =
            vec!["apps/client/dist".into(), "build\\output".into()];
        assert!(validate_verification_manifest(&valid).is_empty());
    }

    #[test]
    fn quiescent_gates_record_matching_fingerprints_and_stay_fresh() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        let mut configured = manifest("rustc");
        configured.gates[0].args = vec!["--version".into()];
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&configured).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-quiet.md");
        fs::write(&spec, "---\nid: SPEC-quiet\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        let approval = dir.path().join("local/approval.json");
        approve_verification_manifest(dir.path(), &approval).unwrap();

        let report = execute_spec_verification(dir.path(), &spec, &approval).unwrap();
        assert!(!report.invalidated);
        assert!(report.invalidation_reason.is_none());
        assert_eq!(
            report.workspace_fingerprint_before,
            report.workspace_fingerprint
        );
        assert!(report.all_expectations_met);
        let document = Document::parse(&fs::read_to_string(&spec).unwrap()).unwrap();
        assert_eq!(
            transcript_state_for_document(dir.path(), &document),
            TranscriptState::GeneratedFresh
        );
    }

    fn sample_transcript(fingerprint: &str) -> (String, String) {
        let without_hash = render_transcript(
            "manifest",
            fingerprint,
            fingerprint,
            "contract",
            &[],
            None,
            None,
        );
        let hash = hex_digest(without_hash.as_bytes());
        let transcript = render_transcript(
            "manifest",
            fingerprint,
            fingerprint,
            "contract",
            &[],
            None,
            Some(&hash),
        );
        (transcript, hash)
    }

    #[test]
    fn generated_transcript_preserves_hand_authored_evidence() {
        let body = "## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n\n### Verification performed\nManual verification summary.\n";
        let (transcript, _) = sample_transcript("first");
        let updated = replace_transcript(body, &transcript).unwrap();

        assert!(updated.contains("$ manual-check\nmanual result"));
        assert!(updated.contains(GENERATED_TRANSCRIPT_START));
        assert!(updated.contains(GENERATED_TRANSCRIPT_END));
        assert!(updated.contains("### Verification performed\nManual verification summary."));
    }

    #[test]
    fn generated_transcript_updates_only_its_managed_region() {
        let body = "## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n";
        let (first, _) = sample_transcript("first");
        let once = replace_transcript(body, &first).unwrap();
        let (second, _) = sample_transcript("second");
        let twice = replace_transcript(&once, &second).unwrap();

        assert!(twice.contains("$ manual-check\nmanual result"));
        assert!(!twice.contains("workspace-fingerprint: first"));
        assert!(twice.contains("workspace-fingerprint: second"));
        assert_eq!(twice.matches(GENERATED_TRANSCRIPT_START).count(), 1);
        assert_eq!(twice.matches(GENERATED_TRANSCRIPT_END).count(), 1);
    }

    #[test]
    fn legacy_generated_transcript_is_migrated_without_losing_manual_evidence() {
        let (legacy, _) = sample_transcript("legacy");
        let body = format!(
            "## Implementation evidence\n\n### Verification transcript\n\n{legacy}\n```text\n$ manual-after-legacy\nmanual result\n```\n"
        );
        let (replacement, _) = sample_transcript("replacement");
        let updated = replace_transcript(&body, &replacement).unwrap();

        assert!(updated.contains("$ manual-after-legacy\nmanual result"));
        assert!(!updated.contains("workspace-fingerprint: legacy"));
        assert!(updated.contains("workspace-fingerprint: replacement"));
    }

    #[test]
    fn verification_merge_uses_the_latest_spec_body() {
        let dir = tempdir().unwrap();
        let spec_dir = dir.path().join(".lmbrain/specs/working");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec = spec_dir.join("SPEC-001.md");
        fs::write(
            &spec,
            "---\nid: SPEC-001\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Changes made\n\ninitial\n\n### Verification transcript\n",
        )
        .unwrap();
        let canonical = spec.canonicalize().unwrap();
        let latest = fs::read_to_string(&spec)
            .unwrap()
            .replace("initial", "initial\nlate agent edit");
        fs::write(&spec, latest).unwrap();
        let (transcript, hash) = sample_transcript("workspace");

        write_verification_transcript(
            dir.path(),
            &canonical,
            "SPEC-001",
            &["sample".into()],
            &transcript,
            &hash,
            "workspace",
        )
        .unwrap();

        let updated = fs::read_to_string(spec).unwrap();
        assert!(updated.contains("late agent edit"));
        assert!(updated.contains(GENERATED_TRANSCRIPT_START));
    }

    #[test]
    fn verification_does_not_recreate_a_spec_moved_during_gate_execution() {
        let dir = tempdir().unwrap();
        let working = dir.path().join(".lmbrain/specs/working");
        let review = dir.path().join(".lmbrain/specs/review");
        fs::create_dir_all(&working).unwrap();
        fs::create_dir_all(&review).unwrap();
        let spec = working.join("SPEC-001.md");
        fs::write(
            &spec,
            "---\nid: SPEC-001\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n",
        )
        .unwrap();
        let canonical = spec.canonicalize().unwrap();
        let moved = review.join("SPEC-001.md");
        fs::rename(&spec, &moved).unwrap();
        let (transcript, hash) = sample_transcript("workspace");

        let error = write_verification_transcript(
            dir.path(),
            &canonical,
            "SPEC-001",
            &["sample".into()],
            &transcript,
            &hash,
            "workspace",
        )
        .unwrap_err();

        assert!(matches!(
            error,
            VerificationError::ConcurrentModification(_)
        ));
        assert!(!spec.exists());
        assert!(moved.exists());
    }

    #[test]
    fn verification_does_not_write_when_the_gate_contract_changes() {
        let dir = tempdir().unwrap();
        let spec_dir = dir.path().join(".lmbrain/specs/working");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec = spec_dir.join("SPEC-001.md");
        let changed = "---\nid: SPEC-001\nstatus: working\nverification_gates: [replacement]\n---\n\n## Implementation evidence\n\n### Verification transcript\n\n```text\n$ manual-check\nmanual result\n```\n";
        fs::write(&spec, changed).unwrap();
        let canonical = spec.canonicalize().unwrap();
        let (transcript, hash) = sample_transcript("workspace");

        let error = write_verification_transcript(
            dir.path(),
            &canonical,
            "SPEC-001",
            &["original".into()],
            &transcript,
            &hash,
            "workspace",
        )
        .unwrap_err();

        assert!(matches!(
            error,
            VerificationError::ConcurrentModification(_)
        ));
        assert_eq!(fs::read_to_string(spec).unwrap(), changed);
    }

    #[test]
    fn missing_local_approval_fails_closed() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/specs/working")).unwrap();
        fs::write(
            dir.path().join(VERIFICATION_MANIFEST_PATH),
            toml::to_string(&manifest("rustc")).unwrap(),
        )
        .unwrap();
        let spec = dir.path().join(".lmbrain/specs/working/SPEC-001.md");
        fs::write(&spec, "---\nid: SPEC-001\nstatus: working\nverification_gates: [sample]\n---\n\n## Implementation evidence\n\n### Verification transcript\n").unwrap();
        assert!(matches!(
            execute_spec_verification(dir.path(), &spec, &dir.path().join("missing.json")),
            Err(VerificationError::ApprovalRequired)
        ));
    }

