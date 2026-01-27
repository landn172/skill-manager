cask "skill-manager" do
  version "0.3.0"
  sha256 "replace_this_with_actual_sha256_checksum"

  url "https://github.com/landn172/skill-manager/releases/download/v#{version}/Skill_Manager_#{version}_universal.dmg"
  name "Skill Manager"
  desc "Manage local AI agents and skills"
  homepage "https://github.com/landn172/skill-manager"

  app "Skill Manager.app"

  zap trash: [
    "~/Library/Application Support/com.jy.skill-manager",
    "~/Library/Caches/com.jy.skill-manager",
    "~/Library/Preferences/com.jy.skill-manager.plist",
    "~/Library/Saved Application State/com.jy.skill-manager.savedState",
  ]
end
