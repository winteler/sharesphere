welcome-to-sharesphere = Bienvenue sur ShareSphere!
markdown-help-1 = Pour formatter votre contenu, le mode 'Markdown' doit être activé avec le bouton suivant:{ " " }
markdown-help-2 = Quand le mode 'Markdown' est activé, votre saisie sera interprété avec le format{ " " }
markdown-help-3 =
     (avec l'addition de 'Spoilers') et une prévisualisation de votre contenu sera affichée.
    Des boutons de formatage rapide sont également disponibles pour que vous n'ayez pas à mémoriser la syntaxe GFM !
    Finalement, le formatage 'Spoiler' peut être généré en ajoutant '||' de chaque côté de votre texte ou
    en le sélectionnant et en cliquant le bouton de formatage 'Spoiler':{ " " }

invalid-link = Lien invalide
invalid-domain-name = Nom de domaine invalide
invalid-video-format = Votre navigateur ne prend pas en charge ce format vidéo.

internal-error-message = Une erreur s'est produite.
not-authenticated-message = Prière de vous authentifier.
authentication-failed-message = Désolé, nous avons des difficultés à vous authentifier.
not-authorized-message = Vous êtes dans une zone restreinte, ne résistez pas.
sphere-ban-until-message = Vous êtes banni de cette sphère jusqu'au
permanent-sphere-ban-message = Vous êtes définitivement banni de cette sphere.
global-ban-until-message = Vous êtes banni de ShareSphere jusqu'au
permanent-global-ban-message = Vous êtes définitivement banni de ShareSphere.
bad-request-message = Désolé, nous n'avons pas compris votre requête.
unavailable-message = Désolé, il y a du bruit sur la ligne.
not-found-message = Il n'y a rien ici.
payload-too-large-message = Le fichier dépasse la limite de  { $byte_limit } Bytes.

clipboard-error-message = API presse-papiers non-supporté par votre navigateur.
copy-link-to-clipboard-message = Lien copié dans le presse-papiers.

refresh = Rafraîchir
cancel = Annuler
submit = Soumettre
login = Se connecter
pinned = Épinglé
delete-warning = Cette action est irréversible.

sphere-banner = Bannière de la sphère

time-seconds-short = s
time-minutes-short = m
time-hours-short = h
time-days-short = j
time-months-short = mo
time-years-short = a

time-seconds = {$count ->
    [one] {$count} seconde
    *[other] {$count} secondes
}
time-minutes = {$count ->
   [one] {$count} minute
   *[other] {$count} minutes
}
time-hours = {$count ->
   [one] {$count} heure
   *[other] {$count} heures
}
time-days = {$count ->
   [one] {$count} jour
   *[other] {$count} jours
}
time-months = {$count} mois
time-years = {$count ->
   [one] {$count} année
   *[other] {$count} années
}