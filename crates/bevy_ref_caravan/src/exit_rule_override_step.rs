use proc_macro::*;
use proc_macro::token_stream::IntoIter as TokenIter;

use crate::common::collect_until_punct::*;
use crate::construction_step::construction_step;
use crate::entity_step::{entity_step_entrance, EntityWildcard};
use crate::syntax_in::{EXIT_RULE_DELIMITER, INTO_NEXT, LINE_BREAK, NEXT};

enum OverrideNext {
    IntoNext,
    Escape,
}

// Exits into construction step
pub(crate) fn exit_rule_override_step(
    mut caravan: TokenIter, 
    package: TokenStream,
    exit_rule: &TokenStream,
    is_nested: bool,

    entity_clause: (EntityWildcard, Vec<TokenTree>),
    query_clause: Vec<TokenTree>,
    bindings_clause: Vec<TokenTree>,
    contains_mut: bool,
) -> Result<(TokenIter, TokenStream), ()> {
    let Some(token) = caravan.next() else {
        return Err(())
    };

    let (mut caravan, override_rule, next) = match token {
        TokenTree::Group(group) => {
            // Validate delimiter
            if group.delimiter() != EXIT_RULE_DELIMITER {
                return Err(())
            }

            // Collect group's tokens.
            let exit_rule = group.stream();

            let (caravan, next) = match validate_override_end(caravan, is_nested) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            };

            (caravan, exit_rule, next)
        },
        _ => {
            let (caravan, exit_rule, next) = match collect_until_override_end(caravan, Vec::new(), is_nested) {
                Ok(ok) => ok,
                Err(err) => return Err(err),
            };

            let exit_rule = TokenStream::from_iter(exit_rule.into_iter());

            (caravan, exit_rule, next)
        }
    };

    let package = match construction_step(package, &override_rule, entity_clause, query_clause, bindings_clause, contains_mut) {
        Ok(ok) => ok,
        Err(err) => return Err(err),
    };

    match next {
        OverrideNext::IntoNext => {
            let Some(current) = caravan.next() else {
                return Err(())
            };

            return entity_step_entrance(caravan, package, exit_rule, is_nested, true, current);
        },
        OverrideNext::Escape => {
            if !is_nested {
                return Ok((caravan, package))
            }

            let Some(current) = caravan.next() else {
                return Err(())
            };

            return entity_step_entrance(caravan, package, exit_rule, is_nested, false, current);
        },
    }
}

fn validate_override_end(
    mut caravan: TokenIter, 
    is_nested: bool,
) -> Result<(TokenIter, OverrideNext), ()> {
    let token = caravan.next();
    let Some(token) = token else { 
        return Ok((caravan, OverrideNext::Escape))
    };

    let TokenTree::Punct(token) = token else { // Is Punct?
        return Err(())
    };

    // Is valid singular token?
    match is_nested {
        true => {
            if token == NEXT { // For nested the NEXT symbol is valid.
                return Ok((caravan, OverrideNext::Escape))
            }
        },
        false => {
            if token == LINE_BREAK { // For un-nested the LINE_BREAK symbol is valid.
                return Ok((caravan, OverrideNext::Escape))
            }
        },
    }

    // Is INTO_NEXT punct combo?
    let (results, caravan, _) = match_one_punct_combo(INTO_NEXT.iter(), caravan, token, Vec::new());
    match results {
        PunctMatch::Matching => return Ok((caravan, OverrideNext::IntoNext)),
        _ => {
            return Err(())
        },
    }
}

// Basically the same thing as collect until bindings end
fn collect_until_override_end(
    mut caravan: TokenIter, 
    mut output: Vec<TokenTree>,
    is_nested: bool,
) -> Result<(TokenIter, Vec<TokenTree>, OverrideNext), ()> {
    let token = caravan.next();
    let Some(token) = token else { // Expect to be un-nested or else throw an error.
        return Ok((caravan, output, OverrideNext::Escape))
    };

    let TokenTree::Punct(token) = token else { // Is Punct?
        output.push(token);
        return collect_until_override_end(caravan, output, is_nested) // If not, continue and add token to output.
    };

    // Is valid singular token?
    match is_nested {
        true => {
            if token == NEXT { // For nested the NEXT symbol is valid.
                return Ok((caravan, output, OverrideNext::Escape))
            }
        },
        false => {
            if token == LINE_BREAK { // For un-nested the LINE_BREAK symbol is valid.
                return Ok((caravan, output, OverrideNext::Escape))
            }
        },
    }

    // Is INTO_NEXT punct combo?
    let (results, caravan, output) = match_one_punct_combo(INTO_NEXT.iter(), caravan, token, output);
    match results {
        PunctMatch::Matching => return Ok((caravan, output, OverrideNext::IntoNext)),
        _ => {
            return collect_until_override_end(caravan, output, is_nested) // If not, continue. (token is already added to output because of match_one_punct_combo).
        },
    }
}