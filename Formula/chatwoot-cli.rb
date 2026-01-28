class ChatwootCli < Formula
  desc "Chatwoot CLI"
  homepage "https://github.com/radjathaher/chatwoot-cli"
  version "0.1.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/radjathaher/chatwoot-cli/releases/download/v0.1.1/chatwoot-cli-0.1.1-darwin-aarch64.tar.gz"
      sha256 "c7f7a4bab45a095ed3c3b9004ca4b1c65f0e6ee6b8abbec58760652404d84fe5"
    else
      odie "chatwoot-cli is only packaged for macOS arm64"
    end
  end

  def install
    bin.install "chatwoot"
  end
end
