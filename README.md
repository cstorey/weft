# Weft: composable HTML templating

[![CircleCI](https://circleci.com/gh/cstorey/weft/tree/master.svg?style=svg)](https://circleci.com/gh/cstorey/weft/tree/master)

## Why another templating language?

Weft is inspired by the [Genshi](https://genshi.edgewall.org/wiki/GenshiFaq) templating language. And in the grand tradition of awful naming puns in open source, since [Genshi means Thread](https://genshi.edgewall.org/wiki/GenshiFaq#WhyisitcalledGenshi), we're calling this Weft, after the base thread used in weaving.

We have a few other main goals, too.

### Composable templates

It should be as easy to compose fragments of template as you do data structures in Rust. For example, many languages use Jinja 2 style [template inheritance](http://jinja.pocoo.org/docs/2.10/templates/#template-inheritance), however this doesn't really feel all that parsimonious with the way rust manages composition, and and usually means that the only way to verify something using one of these templates is via rendering the entire page.

### [Language oriented](http://langsec.org/) design

HTML templating often involves a lot of ad-hoc composition of different languages in different context, such as HTML markup itself, URL parameters, scripts, &c. A lot of engines rely on knowing what escaping to apply in which context in order to avoid content injection attacks. Whilst we can't solve all of these problems at once, we at least aim to ensure that the output markup is correctly formed as much as we can from end to end.

### Attribute based

Many markup langauges involve adding an ad-hoc layer of markup on top of a markup language in order to control what and how content gets rendered. XSLT largely manages to avoid this problem by defining the content transformation in terms of the base markup language itself (XML), but but suffers from being far more general than we need for a template language, and thus can be very verbose.

The idea of using _attributes_ to control how markup is rendered comes from [Zope's Template Attribute Language](https://zope.readthedocs.io/en/latest/zope2book/AppendixC.html) (via Genshi). This seems to work well since it's very unobtrusive, and means that templates render naturally when viewed in a browser.

The post [In search of a Pythonic, XML-based Templating Language](https://tomayko.com/blog/2004/pythonic-xml-based-templating-language) also explains a lot of these ideas very well.