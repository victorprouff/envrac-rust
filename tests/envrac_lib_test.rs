

#[cfg(test)]
mod tests {
    use std::fs;
    use envrac_rust::{execute};

    #[test]
    fn it_works_too() {
        let nom_fichier = "tests/todoist_output.json";
        println!("Dans le fichier : {}", nom_fichier);

        let contenu = fs::read_to_string(nom_fichier)
            .expect("Quelque chose s'est mal passé lors de la lecture du fichier");

        execute(&*contenu).expect("TODO: panic message");

    }
}
