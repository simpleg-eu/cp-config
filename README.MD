# Configuration provider

`cp-config` is a configuration provider for microservices of all kind.

## How to build

### MacOS

First of all we must have `brew` installed.

We install the `pkg-config`:
`brew install pkg-config`

After that we install `openssl`:
`brew install openssl@3.0`

Once `openssl` is installed, we follow the instructions given by the installation
so we add `openssl` to our path:
`echo 'export PATH="/opt/homebrew/opt/openssl@3.0/bin:$PATH"' >> ~/.zshrc`

Finally, we build the project by using:
`cargo build --release`

## How to deploy locally

### Linux / MacOS

In order to run a local instance of `cp-config`, we must have `Docker Desktop`already installed on
our machine. Once you have `docker` installed, just run following command:
`docker compose up`