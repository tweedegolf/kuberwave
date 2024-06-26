# Kuberwave

Conveniently generate and deploy Kubernetes projects for real production applications.

Kuberwave provides:
* A relatively compact form to write deployments in.
* Frequently occurring patterns such as ingress and certificate definitions.
* Multi-environment inventories similar to Ansible.
* A convenient method of storing secrets in your repositories with ansible-vault.
* Checks whether you have provided all required environment variables.
* Locally deploy your projects in a reproducible manner.
* Conveniently deploy on your CI servers.

```
# ./target/debug/kuberwave -h
kuberwave 0.1.0
generate Kubernetes configurations from the commandline

USAGE:
    kuberwave [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    cluster-generate    Generates a cluster configuration and writes to a directory
    deploy              Deploys a configuration to the current cluster
    generate            Generates a configuration and writes to a directory
    help                Prints this message or the help of the given subcommand(s)
```

## Projects

### Generate

```
# ./target/debug/kuberwave generate -h
kuberwave-generate
Generates a configuration and writes to a directory

USAGE:
   kuberwave generate [FLAGS] [OPTIONS] <manifest-path>

FLAGS:
   -d, --dry-run    Do not actually write the configuration
   -h, --help       Prints help information
   -V, --version    Prints version information

OPTIONS:
   -i, --inventory <inventory-path>    Path to inventory file
   -o, --output <output-path>          Path to output directory [default: ./result]

ARGS:
   <manifest-path>    Path to manifest file
```

### Deploy
Directly deploy to a Kubernetes cluster.
Deploys to `kubectl` default cluster or to the default cluster specified with `--kubeconfig`.
Optionally you can authenticate to the cluster with a separate `--token`.

This command is primarily designed to be used by Continuous Integration (CI) environments.

```
# ./target/debug/kuberwave deploy -h
kuberwave-deploy
Deploys a configuration to the current cluster

USAGE:
    kuberwave deploy [FLAGS] [OPTIONS] <manifest-path>

FLAGS:
    -d, --dry-run    Do not actually write the configuration
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --inventory <inventory-path>      Path to inventory file
    -c, --kubeconfig <kubeconfig-path>    Path to kubeconfig file
    -t, --token <token-path>              Path to token file, encrypted with SECRET

ARGS:
    <manifest-path>    Path to manifest file
```

## Inspect serviceaccount privileges
An admin can inspect the privileges handed out to all service account *per namespace* using the following invocation or similar:

```kubectl auth can-i --as system:serviceaccount:default:example-ci --list -n example-production```

Currently we provide two clusterroles via clustergenerate:
* *role-all*: a role that gives all permissions. When bound as a RoleBinding for a specific namespace, will only grant all permissions for that namespace, except for changing more RoleBindings and Roles. When handed out as an ClusterRoleBinding, will grant all permissions.
* *role-view-unprivileged*: a role that gives read-only (read, watch, list) privileges on all objects, except for Secrets.

Generally all service accounts related to users get cluster-wide view-unprivileged access, and global access for concrete namespaces.

## Cluster (generate)

Generates specific cluster definition files such as serviceaccounts and rolebindings.

```
# ./target/debug/kuberwave cluster-generate -h
kuberwave-cluster-generate
Generates a cluster configuration and writes to a directory

USAGE:
    kuberwave cluster-generate [OPTIONS] <manifest-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <output-path>    Path to output directory [default: ./result]

ARGS:
    <manifest-path>    Path to manifest file
```

As a Kubernetes cluster admin you can deploy the resulting files by running:

```
# kubectl apply -Rf ./result/apply
# kubectl auth reconcile -f ./result/auth
```

## Running in docker
You can run `kuberwave` in docker such that it is reproducible, both locally and on a CI-server.
Here is an example script for a typical deployment with an inventory.

You need to set:
* `SECRET`: your ansible vault password
* `ENV`: to the name of your inventory

```
#!/bin/bash
set -e

echo "Deploying $TAG to $ENV"

docker pull ghcr.io/tweedegolf/kuberwave:latest
docker run \
    -e SECRET="$SECRET" \
    -v `pwd`:/app \
    -w /app/deployment \
      ghcr.io/tweedegolf/kuberwave:latest \
      kuberwave deploy \
        --token=./deploy.token \
        --kubeconfig=./kubeconfig.yml \
        --inventory="./inventory/$ENV.yml" \
        ./manifest.yml
```
