class RepoDoctor < Formula
  desc "Local-first repository readiness checker"
  homepage "https://github.com/Kota-Ohno/repo-doctor"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-aarch64-apple-darwin.tar.gz"
      sha256 "TODO"
    else
      url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-x86_64-apple-darwin.tar.gz"
      sha256 "TODO"
    end
  end

  on_linux do
    url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "TODO"
  end

  def install
    bin.install "repo-doctor"
  end

  test do
    assert_match "repo-doctor", shell_output("#{bin}/repo-doctor --help")
  end
end
