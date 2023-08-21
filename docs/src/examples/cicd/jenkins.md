# Jenkins


## Use a Docker image packaging Novops

See [Docker integration](../docker.md) to build a Docker image packaging Novops, then use it in Jenkinsfile such as:

```Jenkinsfile
    agent {
        docker {
            image 'your-image-with-novops'
        }
    }

    stage('Novops') {
        sh '''
            source <(novops load -e dev)
        '''
    }
```

## Install novops on-the-fly

_This method is not recommended. Prefer using an image packaging Novops to avoid unnecessary network load._

Setup a step such as:

```Jenkinsfile
    stage('Novops') {
        sh '''
            curl -L "https://github.com/PierreBeucher/novops/releases/latest/download/novops-X64-Linux.zip" -o novops.zip
            unzip novops.zip
            sudo mv novops /usr/local/bin/novops

            source <(novops load -e dev)
        '''
    }
```

Alternatively, setup a specific version:

```
    environment { 
        NOVOPS_VERSION=0.6.0
    }

    stage('Novops') {
        sh '''
            curl -L "https://github.com/PierreBeucher/novops/releases/download/v${NOVOPS_VERSION}/novops-X64-Linux.zip" -o novops.zip
            unzip novops.zip
            mv novops /usr/local/bin/novops

            source <(novops load -e dev)
        '''
    }
```
