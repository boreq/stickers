module.exports = {
  root: true,
  env: {
    node: true,
  },
  extends: [
    'plugin:vue/vue3-essential',
    '@vue/airbnb',
    '@vue/typescript/recommended',
  ],
  parserOptions: {
    ecmaVersion: 2020,
  },
  rules: {
    'no-console': process.env.NODE_ENV === 'production' ? 'warn' : 'off',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'warn' : 'off',

    // there is some kind of a bug in the linter I understand so the first lint has to be replaced
    // by the second lint
    'no-shadow': 'off',
    '@typescript-eslint/no-shadow': 'warn',

    // really annying and mentions "hoising" in the justification, there is no "hoisting" in
    // typescript so just that fact alone makes me want to disable it
    'no-use-before-define': 'off',

    // I use tridactyl and can click those elements just fine using my
    // keyboard
    'vuejs-accessibility/click-events-have-key-events': 'off',

    // yeah? well, you know, that's just like uh, your opinion, man
    'vue/multi-word-component-names': 'off',

    // for some reason despite adding labels with "for" this keeps getting triggered so I got mad at
    // it and just put headers in
    // todo try to enable those rules
    'vuejs-accessibility/label-has-for': 'off',
    'vuejs-accessibility/form-control-has-label': 'off',

    // what? for ... of ... loops are not allowed? am I going crazy? good job, you made me disable
    // the whole item
    'no-restricted-syntax': 'off',

    // this won't let me create funcs which accept a param to modify it
    'no-param-reassign': 'off',
  },
};
