# Human Optimized Templates
![Build](https://github.com/novakov-alexey/hot/workflows/Build/badge.svg)

Command line tool to render Handlebars templates with values from HOCON file.

Template file: service.yaml

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

Parameters file: params.conf

```hocon
port = 80
port = ${?PORT}
targetPort = 7080
```

```bash
PORT=81 ./ht service.yaml
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

based on Rust crates:
- [handlebars](https://crates.io/crates/handlebars)
- [hocon](https://crates.io/crates/hocon)