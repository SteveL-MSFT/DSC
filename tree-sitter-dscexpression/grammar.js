const SPECIAL_CHARACTERS = [
  '\'', '\'\'',
  '[', ']',
  '[[', '',
  '\\[', '\\]',
  '(', ')'
];

module.exports = grammar({
  name: 'DSCExpression',

  rules: {
    expression: $ => seq('[', $._function, optional($._members), ']'),
    _function: $ => seq($.functionName, '(', optional($._arguments), ')'),
    functionName: $ => /[a-zA-Z]+/,
    _arguments: $ => seq($._argument, repeat(seq(',', $._argument))),
    _argument: $ => choice($._function, $.string, $.number, $.boolean),
    string: $ => seq("'", /[^']*/, "'"),
    number: $ => /\d+/,
    boolean: $ => choice('true', 'false'),
    _members: $ => repeat1($._member),
    _member: $ => seq('.', $.memberName),
    memberName: $ => /[a-zA-Z0-9_-]+/,
  }

});
