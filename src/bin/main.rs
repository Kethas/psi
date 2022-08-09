fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {

    #[test]
    fn test0() {
         use psi::*;
        
         let grammar = psi!{
             start: a;
        
             a: "a",
                (b a);
             b: "b";
         };
        
         let source = "ba".chars();
         let mut parser = Parser::<CharsInput>::new(source);
        
         let result = parser.parse(&grammar).expect("Failed to parse.");
        
         use psi::parse::parsed::ParseTree::*;
         assert_eq!(result,
             Rule("start".to_owned(),
                 vec![
                     Rule("a".to_owned(), vec![
                         Rule("b".to_owned(), vec![
                             Literal("b".to_owned())
                         ]),
                         Rule("a".to_owned(), vec![
                             Literal("a".to_owned())
                         ])
                     ])
                 ]
             )
         )        
    }
}