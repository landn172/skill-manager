import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useMarketplaceStore } from "@/stores/marketplace";

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: {
    getByLabel: vi.fn(),
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn(),
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe("Marketplace Store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe("filteredSkills - Deduplication", () => {
    it("should deduplicate skills by name, prioritizing official sources", () => {
      const store = useMarketplaceStore();

      // Setup sources with different priorities
      store.sources = [
        {
          id: "official-git",
          name: "Official",
          url: "https://github.com/official",
          source_type: "git",
          official: true,
          enabled: true,
        },
        {
          id: "skillsmp",
          name: "SkillsMP",
          url: "https://skillsmp.com",
          source_type: "api",
          official: false,
          enabled: true,
        },
        {
          id: "custom-registry",
          name: "My Registry",
          url: "https://my-registry.com",
          source_type: "registry",
          official: false,
          enabled: true,
        },
      ];

      // Setup skills with duplicates from different sources
      store.skills = [
        {
          name: "react-best-practices",
          description: "From SkillsMP",
          path: "/path/a",
          source_id: "skillsmp",
          source_name: "SkillsMP",
          tags: [],
        },
        {
          name: "react-best-practices",
          description: "From Official (should win)",
          path: "/path/b",
          source_id: "official-git",
          source_name: "Official",
          tags: [],
        },
        {
          name: "react-best-practices",
          description: "From Registry",
          path: "/path/c",
          source_id: "custom-registry",
          source_name: "My Registry",
          tags: [],
        },
        {
          name: "unique-skill",
          description: "Only in SkillsMP",
          path: "/path/d",
          source_id: "skillsmp",
          source_name: "SkillsMP",
          tags: [],
        },
      ];

      const filtered = store.filteredSkills;

      // Should have 2 skills: deduplicated react-best-practices + unique-skill
      expect(filtered.length).toBe(2);

      // The react-best-practices should be from official source
      const reactSkill = filtered.find((s) => s.name === "react-best-practices");
      expect(reactSkill?.source_id).toBe("official-git");
      expect(reactSkill?.description).toBe("From Official (should win)");
    });

    it("should prioritize local over registry over api", () => {
      const store = useMarketplaceStore();

      store.sources = [
        {
          id: "local-source",
          name: "Local",
          url: "/local/path",
          source_type: "local",
          official: false,
          enabled: true,
        },
        {
          id: "registry-source",
          name: "Registry",
          url: "https://registry.com",
          source_type: "registry",
          official: false,
          enabled: true,
        },
        {
          id: "api-source",
          name: "API",
          url: "https://api.com",
          source_type: "api",
          official: false,
          enabled: true,
        },
      ];

      store.skills = [
        {
          name: "my-skill",
          description: "From API",
          path: "/a",
          source_id: "api-source",
          source_name: "API",
          tags: [],
        },
        {
          name: "my-skill",
          description: "From Local (should win)",
          path: "/b",
          source_id: "local-source",
          source_name: "Local",
          tags: [],
        },
        {
          name: "my-skill",
          description: "From Registry",
          path: "/c",
          source_id: "registry-source",
          source_name: "Registry",
          tags: [],
        },
      ];

      const filtered = store.filteredSkills;

      expect(filtered.length).toBe(1);
      expect(filtered[0].source_id).toBe("local-source");
    });

    it("should not deduplicate when a specific source is selected", () => {
      const store = useMarketplaceStore();

      store.sources = [
        {
          id: "source-a",
          name: "Source A",
          url: "https://a.com",
          source_type: "git",
          official: true,
          enabled: true,
        },
        {
          id: "source-b",
          name: "Source B",
          url: "https://b.com",
          source_type: "api",
          official: false,
          enabled: true,
        },
      ];

      store.skills = [
        {
          name: "skill-1",
          description: "From A",
          path: "/a",
          source_id: "source-a",
          source_name: "Source A",
          tags: [],
        },
        {
          name: "skill-1",
          description: "From B",
          path: "/b",
          source_id: "source-b",
          source_name: "Source B",
          tags: [],
        },
      ];

      // Select specific source
      store.selectedSource = "source-b";

      const filtered = store.filteredSkills;

      // Should only show skills from source-b, no deduplication
      expect(filtered.length).toBe(1);
      expect(filtered[0].source_id).toBe("source-b");
    });
  });

  describe("searchQuery filtering", () => {
    it("should filter skills by search query for non-skillsmp sources", () => {
      const store = useMarketplaceStore();

      store.sources = [
        {
          id: "git-source",
          name: "Git",
          url: "https://git.com",
          source_type: "git",
          official: true,
          enabled: true,
        },
      ];

      store.skills = [
        {
          name: "react-hooks",
          description: "React hooks tutorial",
          path: "/a",
          source_id: "git-source",
          source_name: "Git",
          tags: [],
        },
        {
          name: "vue-composables",
          description: "Vue composables guide",
          path: "/b",
          source_id: "git-source",
          source_name: "Git",
          tags: [],
        },
      ];

      store.searchQuery = "react";

      const filtered = store.filteredSkills;

      expect(filtered.length).toBe(1);
      expect(filtered[0].name).toBe("react-hooks");
    });
  });
});
