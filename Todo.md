## Todo

* [ ] `do` expressions
* [ ] default values for function parameters
* [ ] named parameters for function calls
* [x] support for function calls without parentheses, only with one argument (ex. `foo $ "bar"`)

## Bugs

* [ ] (checker) pattern matching doesn't check for exhaustive matches, unreachable matches, or duplicate matches
* [ ] (checker) doesn't check for redundant imports
* [x] (codegen) pattern matching doesn't work properly with lists
* [ ] (runtime) println can't properly print objects