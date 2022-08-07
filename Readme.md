# Psi
> **Warning**
>
> This project is under development. It has many known bugs and missing features.

Psi is a parser aiming to trade performance for ease of use.

## Grammar
A grammar is a collection of rules. 
Each rule is comprised of one or more definitions.
Each definition is a list of atoms. 
An atom is either a literal or a rule. 
> *This is a parallel of terminals and nonterminals, or of tokens and productions.* 

These constructs can be used to represent anything that can be parsed.
This means that a grammar is essentially a declarative assembly language for a Psi parser.

Ideally, there could be a more higher-level rust macro or language to compile into a lower-level grammar.

## Using this crate
Currently, there is no easy way to use this library.

And little documentation...

## Known issues
  - Multiple End RuleParts being generated when compiling a Grammar.
   - Left recursion is impossible currently. This is possible to implement, and kind of necessary because recursion is encouraged.
   - The current parsing implementation is recursive using function calls. This could be optimized in some wa

## License
See LICENSE for details.