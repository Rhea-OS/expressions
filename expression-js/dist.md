# expressions

Parser for the expression language used in the spreadsheet plugin. This document outlines the JavaScript portion of the
library. For Rust see [the Rust crate](../README.md).

## Quickstart

In order to evaluate expressions, three things must happen;

1. A context object containing a [Data Provider](#Data Provider) must be registered
2. The list of all functions, variables and operators needs to be defined
3. The expression needs to be passed into the parser.

## Data Provider

A data provider is an object which converts addresses into values. Addresses are arbitrary text tokens wrapped in
braces. The provider determines their meaning. [See the `repl.ts`](examples/repl.ts) for a feature-complete REPL.

```typescript
import * as expr from 'expression';

class Table implements expr.DataSource {
    query(cx:any, query:string) {
        // Parse the query however you need to.
        // For example, the format `column:row`

        return "Hey!";
    }
}
```

## Context

Next you'll need a context object which holds state and variables and acts as an API to the expression engine. You can
define your own functions, globals and operators here.

```typescript
import * as assert from 'node:assert';
import * as expr from 'expression';

const cx = new expr.Context(new Table())
    .withGlobal("twelve", 12)
    .withGlobal("print", args => console.log(args));

assert.eq(cx.evaluateStr("twelve") == 12);
```