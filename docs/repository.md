# Repository dashboard

The Repository page is a read-only view of the selected workspace's local Git
state and GitHub delivery activity. It does not stage, revert, edit, commit, or
push files.

## Layout and changed files

The dashboard expands on wide desktop windows and collapses to one column when
space is limited. Long branches, remotes, file paths, pull-request titles, and
workflow names are contained within their cards instead of widening the page.
Hover a truncated value to inspect its complete text.

Each changed-file row shows its Git status. Select a row with the mouse or
keyboard to open its unified diff. Staged changes are read from the index;
unstaged and conflicted changes are read from the worktree; untracked files are
shown as additions. Binary files display a non-textual-change notice. Previews
larger than 512 KiB are explicitly truncated, and rendering is capped at 5,000
lines to keep the modal responsive.

The diff modal is read-only and closes from its close button, the Escape key,
or the backdrop. Diff text is rendered as text rather than injected HTML.

## Safety boundary

LMBrain invokes Git directly without a shell, disables external diff and
text-conversion drivers plus color output, and confines requested paths to the selected repository.
Absolute paths, parent traversal, control characters, unsupported diff targets,
and untracked symlink/junction escapes fail closed.

GitHub pull requests and workflow runs remain remote links. The optional GitHub
PAT is stored through the operating-system credential manager and is never
displayed in the dashboard. Builds explicitly enable the keyring crate's native
Windows Credential Manager, Apple Keychain, and Linux Secret Service backends;
keyring 3 otherwise falls back to a non-persistent mock store. Saving reopens
the credential entry and verifies the token can be read before reporting
success. Credential-store failures are surfaced to the caller rather than being
reported as an unconfigured token.
