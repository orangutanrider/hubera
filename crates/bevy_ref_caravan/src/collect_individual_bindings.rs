use proc_macro::*;
use proc_macro::token_stream::IntoIter as TokenIter;

pub(crate) fn collect_individual_bindings(bindings_clause: Vec<TokenTree>) -> Result<Vec<TokenStream>, ()> {
    let caravan = bindings_clause.into_iter();
    let caravan = TokenStream::from_iter(caravan).into_iter();

    let mut collected = Vec::new();
    match entrance(caravan, &mut collected) {
        Ok(_) => {/* Proceed */},
        Err(_) => return Err(()),
    }

    return Ok(collected)
}

fn entrance(
    mut caravan: TokenIter,
    collected: &mut Vec<TokenStream>
) -> Result<(), ()> {
    let Some(token) = caravan.next() else {
        return Ok(())
    };

    match token {
        TokenTree::Group(group) => {
            // Into nested
            let nested_caravan = group.stream().into_iter();
            match entrance(nested_caravan, collected) {
                Ok(_) => {/* Proceed */},
                Err(_) => {return Err(())},
            }

            // Continue across our own scope
            return entrance(caravan, collected)
        },
        TokenTree::Punct(token) => {
            if token == ',' { // If comma error
                return Err(())
            }

            let mut output= Vec::new();
            collect_unchecked(TokenTree::Punct(token), &mut caravan, &mut output);
            let indv_binding_clause = TokenStream::from_iter(output.into_iter());
            collected.push(indv_binding_clause);

            return entrance(caravan, collected)
        },
        _ => {
            let mut output= Vec::new();
            collect_unchecked(token, &mut caravan, &mut output);
            let indv_binding_clause = TokenStream::from_iter(output.into_iter());
            collected.push(indv_binding_clause);

            return entrance(caravan, collected)
        }
    }

}

/// First token is not checked to see whether it is a ',' or not.
fn collect_unchecked(
    current: TokenTree,
    caravan: &mut TokenIter,
    output: &mut Vec<TokenTree>,
) {    
    // Collect
    output.push(current);

    let Some(current) = caravan.next() else {
        return
    };

    return collect(current, caravan, output);
} 

fn collect(
    current: TokenTree,
    caravan: &mut TokenIter,
    output: &mut Vec<TokenTree>,
) {    
    match current {
        TokenTree::Punct(current) => {
            if current == ',' {
                return
            }

            output.push(TokenTree::Punct(current));

            let Some(current) = caravan.next() else {
                return
            };
            return collect(current, caravan, output);
        },
        _ => {/* Proceed */},
    }

    // Collect
    output.push(current);

    let Some(current) = caravan.next() else {
        return
    };

    return collect(current, caravan, output);
} 