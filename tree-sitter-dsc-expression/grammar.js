const SPECIAL_CHARACTERS = [
    '\'', '\'\'',
    '[', ']',
    '[[', '',
    '\\[', '\\]',
    '(', ')'
  ];

  module.exports = grammar({
    name: 'DSC',

    rules: {
        _expression: $ => seq(
            '[',
            $.function,
            ']'
        )
    }

  });
