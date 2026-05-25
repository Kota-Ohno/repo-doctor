class RepoDoctor < Formula
  desc "Local-first repository readiness checker"
  homepage "https://github.com/Kota-Ohno/repo-doctor"
  version "0.1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-aarch64-apple-darwin.tar.gz"
      sha256 "8e6aa25528ee355f528cbc8f4d5a891b4460e3e511e260e13034c85b2046f6a3"
    else
      url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-x86_64-apple-darwin.tar.gz"
      sha256 "8d31acbc4559445fd96dc3243886c69784f218b0f6646b13d93947cd5da01517"
    end
  end

  on_linux do
    url "https://github.com/Kota-Ohno/repo-doctor/releases/download/v#{version}/repo-doctor-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "b4b846bb50c16eeae7acc64cddcd4e79acf997ce69bebb57456d50255ed454c5"
  end

  def install
    bin.install "repo-doctor"
  end

  test do
    assert_match "repo-doctor", shell_output("#{bin}/repo-doctor --help")
  end
end
