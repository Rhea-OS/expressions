# Grammar

```
     Expression := Term [+-] Expression | Term
               Term := Factor [*/%] Term | Factor
             Factor := (Expression) | Literal | Call | List | AssociativeArray
           Call := Name(Expression,*)
        Literal := Name | Number | String | Address
           List := [Expression,*]
AssociatveArray := [Name|String : Expression,*]
```

```
Expression_p = Expression_(p+1) [Op=>P] Expression_p | Expression_(p+1)
```

# Expression Language API

## Data Sources

```rust
pub trait DataSource {
    fn get_value(addr: impl AsRef<Address>) -> Option<Value>;
}
```

A data source is any object which can produce values from addresses.

## Values

```rust
pub struct Value {
    
}
```