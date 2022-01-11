use std::collections::{HashMap, HashSet};
use parser::*;

// -- model

#[derive(Debug, PartialEq)]
struct Food<'a> {
    ingredients: HashSet<&'a str>,
    allergens: HashSet<&'a str>
}


impl<'a> Food<'a> {
    #[allow(dead_code)]
    fn new(ingredients: &[&'a str], allergens: &[&'a str]) -> Self {
        Food {
            ingredients: ingredients.iter().cloned().collect(),
            allergens: allergens.iter().cloned().collect()
        }
    }
}

struct Model<'a> {
    foods: Vec<Food<'a>>,
    ingredients_by_allergen: HashMap<&'a str, HashSet<&'a str>>,
}

impl<'a> Model<'a> {
    fn new(input: &'a str) -> Self {
        let ingredients = one_or_more(whitespace_wrap(word_ref));
        let allergens = word_ref
            .sep_by(match_literal(", "))
            .between(match_literal("(contains "), match_literal(")"));

        let food = pair(ingredients, allergens, |ingredients, allergens| Food {
            ingredients: ingredients.into_iter().collect(),
            allergens: allergens.into_iter().collect()
        });

        let foods = one_or_more(whitespace_wrap(food)).parse(input).unwrap().1;

        Model {
            foods,
            ingredients_by_allergen: HashMap::new()
        }
    }

    fn determine_allergens(&mut self) {
        self.associate_ingredients_with_allergens();
        while !self.is_fully_determined() {
            self.eliminate_duplicate_matches();
        }
    }

    fn associate_ingredients_with_allergens(&mut self) {
        for food in self.foods.iter() {
            for allergen in food.allergens.iter() {
                match self.ingredients_by_allergen.get_mut(allergen) {
                    Some(existing) => {
                        *existing = existing.intersection(&food.ingredients).cloned().collect();
                    }
                    None => {
                        self.ingredients_by_allergen.insert(allergen, food.ingredients.clone());
                    }
                }
            }
        }
    }

    fn is_fully_determined(&self) -> bool {
        self.ingredients_by_allergen.values().all(|ingredients| ingredients.len() < 2)
    }

    fn eliminate_duplicate_matches(&mut self) {
        let determined: HashSet<&'a str> = self.ingredients_by_allergen.values()
            .filter_map(|ingredients|
                if ingredients.len() == 1 { ingredients.iter().next() } else { None }
            ).cloned().collect();

        for ingredients in self.ingredients_by_allergen.values_mut().filter(|ings| ings.len() > 1) {
            *ingredients = ingredients.difference(&determined).cloned().collect();
        }
    }

    fn ingredients_with_allergen(&self) -> HashSet<&'a str> {
        self.ingredients_by_allergen.values().flat_map(|values| values.iter().cloned()).collect()
    }

    fn all_ingredients(&self) -> HashSet<&'a str> {
        self.foods.iter().flat_map(|food| food.ingredients.iter().cloned()).collect()
    }

    fn ingredients_with_no_allergen(&self) -> HashSet<&'a str> {
        self.all_ingredients()
            .difference(&self.ingredients_with_allergen())
            .cloned()
            .collect()
    }

    fn ingredients_alphabetically_by_allergen(&self) -> Vec<&'a str> {
        let mut allergens: Vec<&'a str> = self.ingredients_by_allergen.keys().cloned().collect();
        allergens.sort();
        allergens.iter().filter_map(|a|
            self.ingredients_by_allergen.get(a).and_then(|ings| ings.iter().next())
        ).cloned().collect()
    }
}


// -- problems

fn part1(model: &Model) -> usize {
    model.foods.iter().map(|food|
        food.ingredients.intersection(&model.ingredients_with_no_allergen()).count()
    ).sum()
}

fn part2(model: &Model) -> String {
    model.ingredients_alphabetically_by_allergen().join(",")
}

fn main() {
    let input = include_str!("input.txt");
    let mut model = Model::new(input);
    model.determine_allergens();

    println!("part 1 {}", part1(&model));
    println!("part 2 {}", part2(&model));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> &'static str {
        "mxmxvkd kfcds sqjhc nhms (contains dairy, fish)
         trh fvjkl sbzzf mxmxvkd (contains dairy)
         sqjhc fvjkl (contains soy)
         sqjhc mxmxvkd sbzzf (contains fish)"
    }

    #[test]
    fn test_parser() {
        let model = Model::new(test_input());
        assert_eq!(model.foods, vec![
            Food::new(
                &["mxmxvkd", "kfcds", "sqjhc", "nhms"],
                &["dairy", "fish"]
            ),
            Food::new(
                &["trh", "fvjkl", "sbzzf", "mxmxvkd"],
                &["dairy"]
            ),
            Food::new(
                &["sqjhc", "fvjkl"],
                &["soy"]
            ),
            Food::new(
                &["sqjhc", "mxmxvkd", "sbzzf"],
                &["fish"]
            )
        ]);
    }

    #[test]
    fn test_part1() {
        let mut model = Model::new(test_input());
        model.determine_allergens();
        assert_eq!(part1(&model), 5);
    }

    #[test]
    fn test_part2() {
        let mut model = Model::new(test_input());
        model.determine_allergens();
        assert_eq!(part2(&model), "mxmxvkd,sqjhc,fvjkl");
    }
}
