#let key_value_table(..key_value_pairs) = [
  #let cells = key_value_pairs.pos().map(elem => ([#elem.at(0):#h(5mm)], elem.at(1))).flatten()
  #grid(columns: (auto, auto), gutter: 0.3em, ..cells)
]


#let setup(title: [PLACEHOLDER], authors: (("PLACEHOLDER", 123456), ("PLACEHOLDER", 123456)), content) = [
  #set text(size: 11pt, font: "TeX Gyre Termes", lang: "en")
  #set page(
    paper: "a4",
    margin: ("top": 25mm, "left": 20mm, "right": 20mm, "bottom": 25mm),
    columns: 2,
    footer: align(center, context counter(page).display("1 / 1", both: true))
  )
  #set par(justify: true, first-line-indent: 1.0em, leading: 0.65em, spacing: 0.65em)
  #set heading(numbering: "1.1")

  #place(
    top + center,
    scope: "parent",
    float: true
  )[
    #grid(columns: (1fr, auto), align: (left, right),
      align(horizon, key_value_table(
        ([Studiengang Kurs], [TINF22IT1]),
        ([Vorlesung], [Moderne Konzepte der Informatik]),
        ([Betreuer], [Bauer, Johannes, Prof. Dr.-Ing.]),
        ([Abgabedatum], [#text(red)[TODO]]),
      )),
      image("images/dhbw_icon.png", height: 4em)
    )
    #v(2em)
    #align(center, text(size: 20pt, weight: "bold", title))
    #authors.map(((name, nr)) => [#name (#nr)]).intersperse([,#h(0.5em)]).join()

  ]

  // Copied and adapted from https://github.com/typst/templates/blob/main/charged-ieee/lib.typ
  #show heading: it => {
    // Find out the final number of the heading counter.
    let levels = counter(heading).get()
    let deepest = if levels != () {
      levels.last()
    } else {
      1
    }

    if it.level == 1 {
      // First-level headings are centered smallcaps.
      set align(center)
      show: block.with(above: 13pt, below: 12.75pt, sticky: true)
      show: smallcaps
      set text(size: 11pt)
      if it.numbering != none {
        numbering("I.", deepest)
        h(7pt, weak: true)
      }
      it.body
    } else if it.level == 2 {
      // Second-level headings are run-ins.
      set par(first-line-indent: 0pt)
      set text(size: 11pt, style: "italic", weight: "regular")
      show: block.with(spacing: 10pt, sticky: true)
      if it.numbering != none {
        numbering("A.", deepest)
        h(7pt, weak: true)
      }
      it.body
    } else [
      // Third level headings are run-ins too, but different.
      #if it.level == 3 {
        numbering("a)", deepest)
        [ ]
      }
      _#(it.body):_
    ]
  }

  #content
]
