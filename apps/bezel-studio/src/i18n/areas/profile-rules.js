// New profile rule summaries; the dialog's existing prose stays canonical.
export const PREFIXES = Object.freeze(['profileRules.']);
export default {
  en: {
    'profileRules.sensor': 'when {key} is {comparison} {value} for {seconds} s',
    'profileRules.above': 'above', 'profileRules.below': 'below',
    'profileRules.time': 'on {days}, from {from} to {to}',
    'profileRules.everyDay': 'every day',
    'profileRules.day0': 'Monday', 'profileRules.day1': 'Tuesday', 'profileRules.day2': 'Wednesday',
    'profileRules.day3': 'Thursday', 'profileRules.day4': 'Friday', 'profileRules.day5': 'Saturday', 'profileRules.day6': 'Sunday',
    'profileRules.idle': 'after {minutes} minutes without input',
    'profileRules.locked': 'when the PC is locked',
  },
  'pt-BR': {
    'profileRules.sensor': 'quando {key} fica {comparison} de {value} por {seconds} s',
    'profileRules.above': 'acima', 'profileRules.below': 'abaixo',
    'profileRules.time': 'em {days}, de {from} até {to}',
    'profileRules.everyDay': 'todos os dias',
    'profileRules.day0': 'segunda-feira', 'profileRules.day1': 'terça-feira', 'profileRules.day2': 'quarta-feira',
    'profileRules.day3': 'quinta-feira', 'profileRules.day4': 'sexta-feira', 'profileRules.day5': 'sábado', 'profileRules.day6': 'domingo',
    'profileRules.idle': 'após {minutes} minutos sem atividade',
    'profileRules.locked': 'quando o PC está bloqueado',
  },
  it: {
    'profileRules.sensor': 'quando {key} resta {comparison} {value} per {seconds} s',
    'profileRules.above': 'sopra', 'profileRules.below': 'sotto',
    'profileRules.time': '{days}, dalle {from} alle {to}',
    'profileRules.everyDay': 'ogni giorno',
    'profileRules.day0': 'lunedì', 'profileRules.day1': 'martedì', 'profileRules.day2': 'mercoledì',
    'profileRules.day3': 'giovedì', 'profileRules.day4': 'venerdì', 'profileRules.day5': 'sabato', 'profileRules.day6': 'domenica',
    'profileRules.idle': 'dopo {minutes} minuti di inattività',
    'profileRules.locked': 'quando il PC è bloccato',
  },
};
