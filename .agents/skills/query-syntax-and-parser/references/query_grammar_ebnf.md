# EBNF Grammar for Find-Me Search Queries

Formal grammar specification for tokenizing and parsing search queries.

---

## EBNF Specification

```ebnf
Query       ::= Clause* ;
Clause      ::= Token | NegatedClause | Group ;
NegatedClause ::= '!' Clause ;
Group       ::= '(' Query ')' ;

Token       ::= ExtFilter
              | SizeFilter
              | DateFilter
              | PathFilter
              | AttribFilter
              | RegexFilter
              | PlainTerm ;

ExtFilter    ::= 'ext:' ExtensionList ;
ExtensionList ::= Identifier ( ( ';' | ',' ) Identifier )* ;

SizeFilter   ::= 'size:' ( SizeComparison | SizeRange ) ;
SizeComparison ::= ( '>' | '<' | '>=' | '<=' | '=' )? Number SizeUnit? ;
SizeRange    ::= Number SizeUnit? '..' Number SizeUnit? ;
SizeUnit     ::= 'b' | 'kb' | 'mb' | 'gb' | 'k' | 'm' | 'g' ;

DateFilter   ::= 'dm:' ( DateKeyword | DateRange | DateValue ) ;
DateKeyword  ::= 'today' | 'yesterday' | 'lastweek' | 'thisweek' | 'thismonth' ;
DateValue    ::= Digit{4} '-' Digit{2} '-' Digit{2} ;
DateRange    ::= DateValue '..' DateValue ;

PathFilter   ::= 'path:' PathString ;
AttribFilter ::= 'attrib:' AttribFlags ;
AttribFlags  ::= ( 'd' | 'h' | 'x' | 'l' )+ ;

RegexFilter  ::= 'regex:' RegularExpressionString ;
PlainTerm    ::= QuotedString | Word ;

QuotedString ::= '"' [^"]* '"' ;
Word         ::= [^ \t\r\n:!()]+ ;
```

---

## Parsing Guarantees (Invariant)
1. **Zero Panics**: The parser MUST NEVER panic on arbitrary Unicode or malformed queries.
2. **Partial Recovery**: A malformed operator (e.g. `size:abc`) gracefully degrades to a plain search term rather than throwing a fatal error.
3. **Linear Time**: Lexing and AST construction execute in $O(N)$ where $N$ is query length in bytes.
