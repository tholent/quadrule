# Copyright the quadrule authors. Licensed Apache-2.0.
#
# The Homebrew cask for Quadrule.app (quadrule ADR-025). Its source is packaging/homebrew/quadrule.rb
# in github.com/tholent/quadrule; on each release, that repository's release workflow (job "Homebrew
# tap") fills in the version and the .dmg's sha256 and pushes it to Casks/quadrule.rb in
# github.com/tholent/homebrew-tap. Edit the source, never the tap's copy: the next release
# overwrites it.
cask "quadrule" do
  version "@VERSION@"
  sha256 "@SHA256@"

  url "https://github.com/tholent/quadrule/releases/download/v#{version}/quadrule-#{version}-macos-universal.dmg"
  name "Quadrule"
  desc "Vector design tool for Supernote page templates and wallpapers"
  homepage "https://github.com/tholent/quadrule"

  # Every release before 1.0.0 is a GitHub pre-release, which the plain :github_releases strategy
  # skips too, so this block keeps pre-releases and drops only drafts.
  livecheck do
    url :url
    strategy :github_releases do |json, regex|
      json.filter_map do |release|
        next if release["draft"]

        release["tag_name"]&.[](regex, 1)
      end
    end
  end

  depends_on :macos

  app "Quadrule.app"

  zap trash: [
    "~/Library/Application Support/org.quadrule.quadrule",
    "~/Library/Application Support/quadrule",
    "~/Library/Saved Application State/io.github.tholent.quadrule.savedState",
  ]

  caveats <<~EOS
    Quadrule is not signed by Apple yet, so macOS warns the first time you open it.
    Open it once and close the warning, then go to System Settings > Privacy & Security
    and click "Open Anyway" next to the message about Quadrule.
  EOS
end
