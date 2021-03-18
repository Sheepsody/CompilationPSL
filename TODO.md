# Fonctionnalités à implementer

## Langage de base

- [x] définir une expression comme une contante entière
- [x] afficher une expression
- [x] un programme est une liste d'instructions
- [x] ajouter les opérations arithmétiques + et - (associativité ?)
- [x] ajouter les parenthèses
- [x] ajouter les opérations \* et / (associativité, précédence ?)
- [x] permettre des commentaires par lignes dans le programme
- [x] permettre des commentaires par zones dans le programme
- [x] déclarer/définir une variable scalaire
- [x] affectation d'une expression à une variable scalaire
- [x] utilisation d'une variable dans une expression
- [x] entrée standard vers un scalaire
- [x] condition if-then-else, condition vrai si entier non nul
- [x] ajouter aussi if-then, sans clause else…
- [x] opérations moins unaire, modulo
- [x] boucle while -- algorithme d'Euclide pour le calcul du PGCD
- [ ] définition de tableaux d'entier de taille constante, fixée à la compilation
- [ ] initialisation d'une variable scalaire avec sa déclaration
- [x] constantes booléennes true et false
- [x] comparaisons entières < <= > >= = !=
- [ ] algorithme du crible d'Ératosthène pour trouver les nombres premiers
- [x] calcul des expression booléennes (et, ou, not)

# Fonctions

- [x] procédures simples : pas de retour, pas de paramètres
- [x] fonctions avec retour et commande de retour d'une valeur (return)
- [x] ajouter des arguments passés par valeur aux procédures et fonctions
- [x] avoir des variables de fonction locales
- [x] déclaration de variable globales explicites
- [x] fonctions récursives : (par exemple factorielle)
- [x] vérifier si possible le nombre d'arguments lors des appels

## Développements avancés

- [ ] construction case choix multiples avec des constantes entières
- [ ] construction elif intermédiaire
- [ ] définitions de constantes symboliques (substituées par le lexer)
- [ ] ajouter un type pointeur sur entier (déclaration, référencement, déréférencement…)
- [ ] passage d'arguments scalaire par référence (par pointeur, modifiables)
- [ ] passage d'arguments tableaux par référence
- [ ] ajouter un opérateur factoriel
- [ ] typage : conversion entier booléen quand nécessaire seulement
- [x] ajouter la déclaration de variables booléennes
- [ ] si des évaluations d'expression sont utilisées plusieurs
- [ ] fois (modulo ? autre ?), utiliser des variables temporaires
- [ ] faire une évaluation des expressions logiques rapide (Vrai OU x est Vrai)
- [ ] initialisation d'un tableau avec sa déclaration
- [ ] ajouter une allocation dynamique de tableaux dans la zone de données
- [ ] implémenter une évaluation partielle des expressions entières
- [ ] vérifier que les accès de tableaux sont dans les bornes déclarées

## Other

- [ ] Draw a clear distinction between expressions (binaryop, unaryop, etc.) & items (assignement, loops, etc.)
