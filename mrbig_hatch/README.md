# MrBig Hatch

## Introduction

The `MrBig Hatch` aims at seamlessly melting the local (containerized) development environment with a distant Kubernetes cluster or serverless platform. As such, there's no more a distinction between the development and production (or staging) environments. When offline, `MrBig Hatch` switches automatically to a local (lightweight) Kubernetes cluster, running, for instance, on [Minikube](https://github.com/kubernetes/minikube), [Kind](https://github.com/kubernetes-sigs/kind), [k3s](https://github.com/rancher/k3s) or [Mikrok8s](https://github.com/ubuntu/microk8s), or a local cloud stack (such as, for instance, [Localstack](https://github.com/localstack/localstack), that provides a local AWS cloud stack).

## References

The `MrBig` project was inspired by several great products, including:

- [DevSpace](https://devspace.cloud/)
- [Okteto](https://okteto.com)
- [Cortexlab](https://cortex.dev)
