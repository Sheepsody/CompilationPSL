# Fonctionnalités à implementer

## Langage de base

- [ ] définir une expression comme une contante entière
- [ ] afficher une expression
- [ ] un programme est une liste d'instructions
- [ ] ajouter les opérations arithmétiques + et - (associativité ?)
- [ ] ajouter les parenthèses
- [ ] ajouter les opérations \* et / (associativité, précédence ?)
- [ ] permettre des commentaires par lignes dans le programme
- [ ] permettre des commentaires par zones dans le programme
- [ ] déclarer/définir une variable scalaire
- [ ] affectation d'une expression à une variable scalaire
- [ ] utilisation d'une variable dans une expression
- [ ] entrée standard vers un scalaire
- [ ] condition if-then-else, condition vrai si entier non nul
- [ ] ajouter aussi if-then, sans clause else…
- [ ] opérations moins unaire, modulo
- [ ] boucle while -- algorithme d'Euclide pour le calcul du PGCD
- [ ] définition de tableaux d'entier de taille constante, fixée à la compilation
- [ ] initialisation d'une variable scalaire avec sa déclaration
- [ ] constantes booléennes true et false
- [ ] comparaisons entières < <= > >= = !=
- [ ] algorithme du crible d'Ératosthène pour trouver les nombres premiers
- [ ] calcul des expression booléennes (et, ou, not)
- [ ] attention, l'opérateur NOT de la machine n'est pas logique mais binaire !
- [ ] fonction système exit(int) qui arrête le programme en laissant la valeur sur la pile

# Fonctions

- [ ] procédures simples : pas de retour, pas de paramètres
- [ ] fonctions avec retour et commande de retour d'une valeur (return)
- [ ] on pourra différencier syntaxiquement les fonctions des procédures
- [ ] ajouter des arguments passés par valeur aux procédures et fonctions
- [ ] avoir des variables de fonction locales
- [ ] déclaration de variable globales explicites
- [ ] fonctions récursives : (par exemple factorielle)
- [ ] sauvegarde dans les piles des variables \textbf{locales} de la fonction\ldots
- [ ] on pourra différencier syntaxiquement la déclaration de ces fonctions
- [ ] vérifier si possible le nombre d'arguments lors des appels

## Développements avancés

- [ ] construction case choix multiples avec des constantes entières
- [ ] construction elif intermédiaire
- [ ] définitions de constantes symboliques (substituées par le lexer)
- [ ] ajouter un type pointeur sur entier (déclaration, référencement, déréférencement…)
- [ ] passage d'arguments scalaire par référence (par pointeur, modifiables)
- [ ] passage d'arguments tableaux par référence
- [ ] ajouter un opérateur factoriel
- [ ] typage : conversion entier booléen quand nécessaire seulement
- [ ] ajouter la déclaration de variables booléennes dont les valeurs sont toujours limitées à 0 ou 1.
- [ ] si des évaluations d'expression sont utilisées plusieurs
- [ ] fois (modulo ? autre ?), utiliser des variables temporaires
- [ ] faire une évaluation des expressions logiques rapide (Vrai OU x est Vrai)
- [ ] initialisation d'un tableau avec sa déclaration
- [ ] ajouter une allocation dynamique de tableaux dans la zone de données
- [ ] (on ne s'occupera pas de libérer la mémoire…)
- [ ] implémenter une évaluation partielle des expressions entières
- [ ] vérifier que les accès de tableaux sont dans les bornes déclarées
