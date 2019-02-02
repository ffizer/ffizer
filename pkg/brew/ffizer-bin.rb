class FfizerBin < Formula
  version '0.9.0'
  desc "ffizer is a files and folders initializer / generator. Create any kind (or part) of project from template."
  homepage "https://github.com/davidB/ffizer"

  if OS.mac?
      url "#{homepage}/releases/download/#{version}/ffizer-#{version}-x86_64-apple-darwin.tar.gz"
      # sha256 ""
  elsif OS.linux?
      url "#{homepage}/releases/download/#{version}/ffizer-#{version}-x86_64-unknown-linux-musl.tar.gz"
      # sha256 ""
  end

  #conflicts_with "ffizer"

  def install
    bin.install "ffizer"
    # man1.install "doc/ffizer.1"

    # bash_completion.install "complete/ffizer.bash"
    # fish_completion.install "complete/ffizer.fish"
    # zsh_completion.install "complete/_ffizer"
  end

  test do
    system "#{bin}/program", "--help"
  end
end
