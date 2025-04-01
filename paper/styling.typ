#let setup(title: [PLACEHOLDER], authors: ("PLACEHOLDER", "PLACEHOLDER"), content) = [
  #title
  #pagebreak()

  #authors.map(author => [#author]).intersperse(h(2em)).join()

  #content
]
