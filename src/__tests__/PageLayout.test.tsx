import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  CardGrid,
  EmptyState,
  PageHeader,
  PageSection,
  PageShell,
  Toolbar,
} from "../components/Shared/PageLayout";

describe("PageShell archetypes", () => {
  it("constrains reading and dense pages with an inner column", () => {
    const { container, rerender } = render(
      <PageShell archetype="reading">
        <p>body</p>
      </PageShell>,
    );
    let shell = container.querySelector("[data-archetype]");
    expect(shell?.getAttribute("data-archetype")).toBe("reading");
    expect(shell?.className).toContain("lm-page--reading");
    expect(shell?.querySelector(".lm-page__inner")).not.toBeNull();

    rerender(
      <PageShell archetype="dense">
        <p>body</p>
      </PageShell>,
    );
    shell = container.querySelector("[data-archetype]");
    expect(shell?.className).toContain("lm-page--dense");
    expect(shell?.querySelector(".lm-page__inner")).not.toBeNull();
  });

  it("leaves full-bleed pages to manage their own panes", () => {
    const { container } = render(
      <PageShell archetype="full">
        <p>pane</p>
      </PageShell>,
    );
    const shell = container.querySelector("[data-archetype='full']");
    expect(shell?.className).toContain("lm-page--full");
    expect(shell?.querySelector(".lm-page__inner")).toBeNull();
  });

  it("keeps caller class names and styles", () => {
    const { container } = render(
      <PageShell archetype="dense" className="custom" style={{ background: "red" }}>
        <p>body</p>
      </PageShell>,
    );
    const shell = container.querySelector("[data-archetype='dense']") as HTMLElement;
    expect(shell.className).toContain("custom");
    expect(shell.style.background).toBe("red");
  });
});

describe("PageHeader", () => {
  it("renders one page title, with optional description and actions", () => {
    render(<PageHeader title="Decisions" description="Records" actions={<button>New</button>} />);
    expect(screen.getByRole("heading", { level: 1, name: "Decisions" })).toBeTruthy();
    expect(screen.getByText("Records")).toBeTruthy();
    expect(screen.getByRole("button", { name: "New" })).toBeTruthy();
  });

  it("omits the description and actions when not supplied", () => {
    const { container } = render(<PageHeader title="Skills" />);
    expect(container.querySelector(".lm-page-header__description")).toBeNull();
    expect(container.querySelector(".lm-page-header__actions")).toBeNull();
  });
});

describe("CardGrid", () => {
  it("uses the approved 360px card minimum by default", () => {
    const { container } = render(
      <CardGrid>
        <div>card</div>
      </CardGrid>,
    );
    const grid = container.querySelector(".lm-card-grid") as HTMLElement;
    expect(grid.getAttribute("data-min-column-width")).toBe("360");
    expect(grid.style.getPropertyValue("--lm-card-min")).toBe("360px");
  });

  it("accepts a narrower minimum where the content allows it", () => {
    const { container } = render(
      <CardGrid minColumnWidth={280}>
        <div>card</div>
      </CardGrid>,
    );
    const grid = container.querySelector(".lm-card-grid") as HTMLElement;
    expect(grid.style.getPropertyValue("--lm-card-min")).toBe("280px");
  });
});

describe("PageSection, Toolbar and EmptyState", () => {
  it("render their content through the shared classes", () => {
    const { container } = render(
      <>
        <PageSection title="Active" description="What needs attention">
          <p>rows</p>
        </PageSection>
        <Toolbar>
          <button>Filter</button>
        </Toolbar>
        <EmptyState>Nothing yet.</EmptyState>
      </>,
    );
    expect(screen.getByRole("heading", { level: 2, name: "Active" })).toBeTruthy();
    expect(container.querySelector(".lm-toolbar")).not.toBeNull();
    expect(container.querySelector(".lm-empty-state")?.textContent).toBe("Nothing yet.");
  });
});
