class ChatwootCli < Formula
  desc "Chatwoot CLI"
  homepage "https://github.com/radjathaher/chatwoot-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/radjathaher/chatwoot-cli/releases/download/v0.1.0/chatwoot-cli-0.1.0-darwin-aarch64.tar.gz"
      sha256 "4c85f90d190cf3b1e9ddc6ca2ca1ed66bfaec856257ed1592e0bffed1efb9e41"
    else
      odie "chatwoot-cli is only packaged for macOS arm64"
    end
  end

  def install
    bin.install "chatwoot"
  end
end
