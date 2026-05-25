class RepoDoctor < Formula
  desc "Local-first repository readiness checker"
  homepage "https://github.com/Kota-Ohno/repo-doctor"
  version "0.1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-aarch64-apple-darwin.tar.gz"
      sha256 "b5d50418c22e748c2269f2ec59b73d08c2ce802167b7709e481ce06361136672"
    else
      url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-x86_64-apple-darwin.tar.gz"
      sha256 "eadda7fef0ad8746ee1ad177fc424cc862daeb6676b22f583137d7d2e7af0737"
    end
  end

  on_linux do
    url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "a77ba5be28a68c218a848991462ff01cc14fe4e102e2bc865727223181bdb7fc"
  end

  def install
    bin.install "repo-doctor"
  end

  test do
    assert_match "repo-doctor", shell_output("#{bin}/repo-doctor --help")
  end
end
