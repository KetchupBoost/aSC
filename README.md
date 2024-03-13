# Intro

This repo is my late version of the [challenge](https://github.com/zanfranceschi/rinha-de-backend-2023-q3/blob/main/README.md). It is about the implementation of an API, that has endpoints to create, consult and find "People". In this tournament, you have to deal with CPU and Memory restriction, each participant has to deliver an API in docker-composer format that uses 1.5 units of CPU and 3 GB of RAM.

## Stack

- Rust
  - Axum (Framework HTTP)
  - SqlX (Async SQL Toolkit with Query Check in COMPILE TIME!!!!)
  - Tokio (Async Runtime non-blockent)
  - PostgreSQL (Relational Data Base)
  - Serde (Serialization/Deseralization)
  - Redis (In Memory Data Base Non Relational )
  - Deadpool-redis (Redis Connection Manager)
- PostGres
- NginX
