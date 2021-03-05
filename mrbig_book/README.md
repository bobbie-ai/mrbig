# MrBig Book

The `MrBig Book` is the project's bible. This is a markdown-based document describing all aspects of MrBig (i.e. architecture, usage, ...).

## Building the MrBig Book's Website

When located at the project's root folder, MrBig Book's website can be generated as follows:

```sh
$ cargo make book-build
```

## Serving the MrBig Book's Website

When contributing to MrBig Book's contents, the book's website can be served with hot-reload feature. Consequently, each time a modification is made to the content, the website is re-generated and updated in the browser. The following command must be used to serve the MrBig Book's website:

```sh
$ cargo make book-serve
```