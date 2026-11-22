/* =========================================================
   RÉGLAGES DU SITE NORÉLIAS — c'est le seul fichier à modifier
   pour les liens de paiement et l'envoi des courriels.
   ========================================================= */
window.NORELIAS = {
  courriel: 'bonjour@norelias.ca',
  telephone: '506-787-0667',

  // Courriel qui reçoit les virements Interac
  interac: 'bonjour@norelias.ca',

  // Liens de paiement Stripe (Stripe > Liens de paiement > Créer).
  // Créez un lien pour CHAQUE montant ci-dessous, puis collez-le entre les guillemets.
  // Tant qu'un lien est vide, la cliente est dirigée vers le virement Interac.
  stripe: {
    essentiel: '',   // acompte de 100 $ (50 % de 200 $)
    signature: '',   // acompte de 150 $ (50 % de 300 $)
    complete:  '',   // acompte de 225 $ (50 % de 450 $)
    noel:      ''    // mini-séance de Noël, paiement complet de 95 $
  },

  // Envoi automatique des demandes par courriel (EmailJS, comme sur academiesere.ca).
  // Laissez vide pour que les demandes s'ouvrent dans l'application courriel de la cliente.
  emailjs: {
    service: '',
    modeleReservation: '',
    modeleContact: '',
    clePublique: ''
  }
};
