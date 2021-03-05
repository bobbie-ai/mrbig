###############################################################################
#                                                                             #
# mrbig/.devcontainer/Dockerfile                                              #
#                                                                             #
# Handcrafted with love by MrBig Mobsters                                     #
# All rights reserved                                                         #
#                                                                             #
#                                                                             #
# Description: Docker image used by VSCode for building a Rust development    #
#              sandbox.                                                       #
# See:         https://code.visualstudio.com/docs/remote/containers           #
###############################################################################


# -----------------------------------------------------------------------------
# SANDBOX STAGE
# -----------------------------------------------------------------------------

FROM rust:1.46.0

# This Dockerfile adds a non-root 'vscode' user with sudo access. However, for Linux,
# this user's GID/UID must match your local user UID/GID to avoid permission issues
# with bind mounts. Update USER_UID / USER_GID if yours is not 1000. See
# https://aka.ms/vscode-remote/containers/non-root-user for details.
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
ARG BOOK_SERVER_PORT=3000
ARG BOOK_WEB_PORT=9999
ARG BAZEL_VERSION=2.0.0
ARG ARGOCD_CLI_VERSION=1.3.6
ARG ARGO_CLI_VERSION=2.2.1
ARG TERRAFORM_VERSION=0.12.20
ARG KUBECTL_VERSION=1.17.2
ARG GLOOCTL_VERSION=1.2.15
ARG HELM_VERSION=3.0.2
ARG MDBOOK_VERSION=0.3.5
ARG DEVTOOLS_VERSION=0.2.0

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Expose mdbook server's port (useful to hot reload the book while editing its content)
EXPOSE ${BOOK_SERVER_PORT} ${BOOK_WEB_PORT}

# Configure apt and install packages
RUN apt-get update \
 && apt-get -y install --no-install-recommends \
    apt-utils \
    dialog \
    git \
    groff \
    iproute2 \
    lldb \
    lsb-release \
    procps \
    socat \
    libgflags-dev \
    libelf-dev \
    libdw-dev \
    cmake \
    binutils-dev \
    2>&1 \
    #
    # Install Rust components
 && rustup update \
 && rustup component add \
    rls \
    rust-analysis \
    rust-src \
    rustfmt \
    clippy \
    #
    # Install Cargo stuffs
 && cargo install \
    cargo-web \
    cargo-make \
    #
    # Create a non-root user to use if preferred - see https://aka.ms/vscode-remote/containers/non-root-user.
 && groupadd --gid $USER_GID $USERNAME \
 && useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -m $USERNAME \
    #
    # [Optional] Add sudo support for the non-root user
 && apt-get install -y sudo \
 && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME\
 && chmod 0440 /etc/sudoers.d/$USERNAME \
    #
    # Install Terraform
 && curl --silent -o "/tmp/terraform.zip" "https://releases.hashicorp.com/terraform/${TERRAFORM_VERSION}/terraform_${TERRAFORM_VERSION}_linux_amd64.zip" \
 && unzip /tmp/terraform.zip \
 && mv terraform /usr/local/bin \
 && rm /tmp/terraform.zip \
    #
    # Install Helm
 && curl --silent -o helm.tar.gz https://get.helm.sh/helm-v${HELM_VERSION}-linux-amd64.tar.gz \
 && tar xf helm.tar.gz \
 && mv linux-amd64/helm /usr/local/bin \
 && rm -rf linux-amd64 helm.tar.gz \
    #
    # Install Bazel
 && curl -LO "https://github.com/bazelbuild/bazel/releases/download/${BAZEL_VERSION}/bazel_${BAZEL_VERSION}-linux-x86_64.deb" \
 && dpkg -i bazel_*.deb \
 && rm -rf bazel_*.deb \
    #
    # Install Argo CD CLI
 && curl --silent -o /usr/local/bin/argocd https://github.com/argoproj/argo-cd/releases/download/v${ARGOCD_CLI_VERSION}/argocd-linux-amd64 \
 && chmod +x /usr/local/bin/argocd \
    #
    # Argo CLI
 && curl -sSL -o /usr/local/bin/argo https://github.com/argoproj/argo/releases/download/v${ARGO_CLI_VERSION}/argo-linux-amd64 \
 && chmod +x /usr/local/bin/argo \
    #
    # Install AWS EKS cluster CLI
 && curl --silent --location "https://github.com/weaveworks/eksctl/releases/download/latest_release/eksctl_$(uname -s)_amd64.tar.gz" | tar xz -C /tmp \
 && mv /tmp/eksctl /usr/local/bin \
    #
    # Install AWS IAM Authenticator tool
 && curl --silent -o "/usr/local/bin/aws-iam-authenticator" https://amazon-eks.s3-us-west-2.amazonaws.com/1.14.6/2019-08-22/bin/linux/amd64/aws-iam-authenticator \
 && chmod +x /usr/local/bin/aws-iam-authenticator \
    #
    # Install AWS CLI v2.x
 && curl --silent -o "/tmp/awscli.zip" "https://d1vvhvl2y92vvt.cloudfront.net/awscli-exe-linux-x86_64.zip" \
 && cd /tmp && unzip awscli.zip && sudo aws/install \
 && rm ./awscli.zip && rm -rf ./aws \
   #
   # Install kubectl CLI
 && curl -SL -o /tmp/kubectl "https://storage.googleapis.com/kubernetes-release/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl" \
 && cd /tmp && chmod +x ./kubectl && sudo mv ./kubectl /usr/local/bin/kubectl \
   #
   # Install glooctl cli.
&& curl -SL -o /tmp/glooctl "https://github.com/solo-io/gloo/releases/download/v${GLOOCTL_VERSION}/glooctl-linux-amd64" \
&& cd /tmp && chmod +x ./glooctl && sudo mv ./glooctl /usr/local/bin/glooctl \
   #
   # Install mdbook tool
 && curl -sSL "https://github.com/rust-lang-nursery/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar xz -C /tmp \
 && chmod 755 /tmp/mdbook \
 && mv /tmp/mdbook /usr/local/bin \
   #
   # Install grpc_cli for testing purposes
 && curl -sSL "https://github.com/bobbie-ai/devcontainer-tools/releases/download/v${DEVTOOLS_VERSION}/grpc_cli.tar.gz" | tar xz -C /tmp \
 && cd /tmp && sudo mv ./grpc_cli /usr/local/bin/grpc_cli \
   #
   # Install kcov for test coverage
 && curl -sSL "https://github.com/bobbie-ai/devcontainer-tools/releases/download/v${DEVTOOLS_VERSION}/kcov.tar.gz" | tar xz -C /tmp \
 && cd /tmp && sudo mv ./kcov /usr/local/bin/kcov \
    #
    # Clean up
 && apt-get autoremove -y \
 && apt-get clean -y \
 && rm -rf /var/lib/apt/lists/*

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=
