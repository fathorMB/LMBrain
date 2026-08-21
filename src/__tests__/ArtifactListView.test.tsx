import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ArtifactListView } from "../components/Shared/ArtifactListView";
import { FilterBar, FilterSelect, FilterSearchInput } from "../components/Shared/FilterBar";

describe("ArtifactListView and FilterBar components", () => {
  it("renders page header and list items within CardGrid", () => {
    const items = [
      { id: "1", title: "Item 1" },
      { id: "2", title: "Item 2" },
    ];

    render(
      <ArtifactListView
        title="Test Artifacts"
        description="A list of artifacts for testing"
        items={items}
        totalCount={items.length}
        renderItem={(item) => (
          <div key={item.id} data-testid="artifact-card">
            {item.title}
          </div>
        )}
      />
    );

    expect(screen.getByRole("heading", { level: 1, name: "Test Artifacts" })).toBeDefined();
    expect(screen.getByText("A list of artifacts for testing")).toBeDefined();
    expect(screen.getAllByTestId("artifact-card")).toHaveLength(2);
  });

  it("shows emptyAllMessage when total count is 0", () => {
    render(
      <ArtifactListView
        title="Empty Artifacts"
        items={[]}
        totalCount={0}
        emptyAllMessage="Nothing here yet."
      />
    );

    expect(screen.getByText("Nothing here yet.")).toBeDefined();
  });

  it("shows emptyFilteredMessage when items are filtered to 0", () => {
    render(
      <ArtifactListView
        title="Filtered Artifacts"
        items={[]}
        totalCount={5}
        emptyFilteredMessage="No results matched your search."
      />
    );

    expect(screen.getByText("No results matched your search.")).toBeDefined();
  });

  it("renders FilterBar with FilterSelect and FilterSearchInput", () => {
    render(
      <FilterBar ariaLabel="Test filters">
        <FilterSelect
          label="Category"
          value="all"
          onChange={() => {}}
          allLabel="All Categories"
          options={["Engine", "UI"]}
        />
        <FilterSearchInput
          label="Search"
          value=""
          onChange={() => {}}
          placeholder="Filter..."
        />
      </FilterBar>
    );

    expect(screen.getByLabelText("Test filters")).toBeDefined();
    expect(screen.getByLabelText("Category")).toBeDefined();
    expect(screen.getByLabelText("Search")).toBeDefined();
  });
});
