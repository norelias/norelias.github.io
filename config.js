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
    noel:      '',   // mini-séance de Noël, paiement complet de 95 $
    personnalise: '' // dépôt de 100 $ du forfait personnalisé
  },

  // DISPONIBILITÉS DU CALENDRIER DE RÉSERVATION
  // Quand une date est prise, ajoutez-la dans « dejaReserve » pour qu'elle disparaisse du calendrier.
  disponibilites: {
    // Heures offertes chaque jour pour les séances
    heures: ['9 h', '10 h 30', '13 h', '14 h 30', '16 h'],
    // Heures des mini-séances de Noël (20 min chacune), offertes seulement en décembre
    heuresNoel: ['9 h', '9 h 30', '10 h', '10 h 30', '11 h', '11 h 30', '13 h', '13 h 30', '14 h', '14 h 30', '15 h', '15 h 30', '16 h', '16 h 30', '17 h'],
    // Nombre de jours minimum avant une réservation
    delaiMinimum: 2,
    // Combien de jours à l'avance on peut réserver
    joursAffiches: 120,
    // Journées complètes fermées (vacances, congés). Format : 'AAAA-MM-JJ'
    joursFermes: ['2026-12-25', '2027-01-01'],
    // Heures déjà réservées. Exemple : '2026-12-05': ['9 h', '10 h 30']
    // Mettez ['tout'] pour bloquer toute la journée.
    dejaReserve: {
    }
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
