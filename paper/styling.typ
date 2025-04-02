#let key_value_table(..key_value_pairs) = [
  #let cells = key_value_pairs.pos().map(elem => ([#elem.at(0):#h(5mm)], elem.at(1))).flatten()
  #grid(columns: (auto, auto), gutter: 1.25em, ..cells)
]


#let setup(title: [PLACEHOLDER], authors: (("PLACEHOLDER", 123456), ("PLACEHOLDER", 123456)), content) = [
  #set text(size: 11pt, font: "TeX Gyre Termes", lang: "en")
  #set page(paper: "a4", margin: 25mm, columns: 2)
  #set par(justify: true, first-line-indent: 1.5em, leading: 0.65em, spacing: 0.85em)

  #place(
    top + center,
    scope: "parent",
    float: true
  )[
    #grid(columns: (1fr, auto), align: (left, right),
      align(horizon, key_value_table(
        ([Vorlesung], [Moderne Konzepte der Informatik]),
        ([Betreuer], [Bauer, Johannes, Prof. Dr.-Ing.]),
        ([Abgabedatum], [#text(red)[TODO]]),
      )),
      image("images/dhbw_icon.png", height: 4em)
    )
    #align(center, text(size: 20pt, weight: "bold", title))
    #authors.map(((name, nr)) => [#name (#nr)]).intersperse([,#h(0.5em)]).join()

  ]

  #content
]
