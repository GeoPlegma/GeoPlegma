---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "GeoPlegma"
  tagline: |
    Implementations and abstractions for
    Discrete Global Grid Systems in Rust

  actions:
    - theme: brand
      text: Get started
      link: /get-started
    - theme: alt
      text: docs.rs
      link: https://docs.rs/geoplegma
    - theme: alt
      text: GitHub
      link: https://github.com/GeoPlegma/GeoPlegma

features:
  - title: Less distortions
    details: |
      Discrete Global Grid Systems (DGGS) tesselate the surface of the earth into zones of equal area,
      minimizing spatial distortions.
  - title: Abstractions
    details: GeoPlegma provides traits and APIs to implement DGGS in Rust.
  - title: Available DGGS
    details: |
      GeoPlegma currently supports the following DGGS: DGGAL, DGGR
  - title: Get involved
    details: |
      GeoPlegma is an evolving platform. Pull Requets are very welcome!
---
