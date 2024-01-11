# BitTorrent Server Con Rust

## Taller de Programación I 
### Facultad de Ingeniería - Universidad de Buenos Aires

Trabajo realizado durante el 1er cuatrimestre del 2022. Este proyecto implementa la funcionalidad de servidor de nuestro BitTorrent, desarrollado en Rust utilizando Cargo. Proporciona una implementación funcional del protocolo BitTorrent, permitiendo la descarga de archivos mediante la red P2P.

Este repositorio es una copia del original, debido a que este es privado y pertenece a la organización de la cátedra. 

### Integrantes 

Nombre |   Email
------ |  -------------
[FEIJOO, Sofia](https://github.com/feijooso) | sfeijoo@fi.uba.ar
[MAZZA RETA, Tizziana](https://github.com/tizziana) | tmazzar@fi.uba.ar
[REINAUDO, Dante](https://github.com/DanteReinaudo) | dreinaudo@fi.uba.ar
[Milhas, Facundo](https://github.com/facundomilhas) | fmilhas@fi.uba.ar


## Estructura del Proyecto

- **`app`**: Contiene la implementación de la interfaz grafica del BitTorrent.
- **`src`**: Contiene las estructuras y componentes esenciales del servidor BitTorrent


## Como usar

* Compila el proyecto utilizando Cargo:
```bash
$ cargo build --release
```

* Ejecuta el Cliente:
```bash
$ cargo run
```

* Ejecuta los tests:
```bash
$ cargo test
```
