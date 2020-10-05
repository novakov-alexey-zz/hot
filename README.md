# Human Optimized Templates
![Build](https://github.com/novakov-alexey/hot/workflows/Build/badge.svg)

Command line tool to render [Handlebars](https://handlebarsjs.com/) templates with values from [HOCON](https://github.com/lightbend/config/blob/master/HOCON.md) file.

Based on Rust crates:
- [handlebars](https://crates.io/crates/handlebars)
- [hocon](https://crates.io/crates/hocon)

## Install

OSX:
```bash
sudo curl -LJ https://github.com/novakov-alexey/hot/releases/download/v0.4.0/ht-macos > /usr/local/bin/ht && chmod +x /usr/local/bin/ht
```
Linux:
```bash
sudo curl -LJ https://github.com/novakov-alexey/hot/releases/download/v0.4.0/ht-linux > /usr/local/bin/ht && chmod +x /usr/local/bin/ht
```

Windows:
```bash
sudo curl -LJ https://github.com/novakov-alexey/hot/releases/download/v0.4.0/ht-windows > ht 
```

## Usage
Create a template file in `templates` directory. 
For example `templates/service.yaml` with below content:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp  
spec:
  selector:
    app: myapp
  ports:
  - port: {{ port }}
    targetPort: {{ targetPort }}
```

Create parameters file `templates/params.conf` with the below content:

```hocon
port = 80
port = ${?PORT}
targetPort = 7080
```
Now run the `ht` command:

```bash
PORT=81 ht -t templates
apiVersion: v1
kind: Service
metadata:
name: myapp
spec:
selector:
 app: myapp
ports:
- port: 81
 targetPort: 7080
``` 

Default template path is `templates`, so below command works as well:

```bash
PORT=81 ht
```
