const esprima = require('esprima');
const walk = require('esprima-walk');

class ForbiddenModuleError extends Error {
  constructor(modules) {
    const prefix = `Forbidden ${modules.length > 1 ? 'modules' : 'module'}: `;
    const message = `${prefix}${modules.length > 1 ? modules.join(', ') : modules}`;
    super(message);
    this.name = 'ForbiddenModuleError';
  }
}

class InfiniteLoopError extends Error {
  constructor() {
    super('Infinite loop detected');
    this.name = 'InfiniteLoopError';
  }
}

const BINARY_OPS = {
  '+': (a, b) => a + b,
  '-': (a, b) => a - b,
  '*': (a, b) => a * b,
  '/': (a, b) => a / b,
  '%': (a, b) => a % b,
  '**': (a, b) => a ** b,
  '|': (a, b) => a | b,
  '&': (a, b) => a & b,
  '^': (a, b) => a ^ b,
  '<<': (a, b) => a << b,
  '>>': (a, b) => a >> b,
  '>>>': (a, b) => a >>> b,
  '==': (a, b) => a == b,
  '!=': (a, b) => a != b,
  '===': (a, b) => a === b,
  '!==': (a, b) => a !== b,
  '<': (a, b) => a < b,
  '<=': (a, b) => a <= b,
  '>': (a, b) => a > b,
  '>=': (a, b) => a >= b,
  '||': (a, b) => a || b,
  '&&': (a, b) => a && b,
  '??': (a, b) => a ?? b,
  'in': (a, b) => a in b,
  'instanceof': (a, b) => a instanceof b,
};

const isForbidden = (module) => {
  const forbiddenRequires =
    ['fs', 'child_process', 'path', 'os', 'http', 'https', 'net', 'tls', 'dns', 'url', 'util', 'vm']
      .map(module => [module, `${module}/promises`, `node:${module}`, `node:${module}/promises`])
      .flat();

  return forbiddenRequires.includes(module) || module === 'unknown';
}

const getVariables = (tokens) => {
  const variables = {};

  walk(tokens, node => {
    if (node.type === 'VariableDeclarator') {
      variables[node.id.name] = node.init.type === 'Literal' ? node.init.value : node.init.name;
    } else if (node.type === 'ForStatement') {
      if (node.init && node.init.type === 'VariableDeclaration') {
        variables[node.init.declarations[0].id.name] = node.init.declarations[0].init.value;
      }
    } else if (node.type === 'ObjectExpression') {
      node.properties.forEach(property => {
        if (property.key.type === 'Identifier') {
          variables[property.key.name] = property.value.name;
        }
      });
    }
  });

  return variables;
}

const parseExternalCode = (code) => {
  const tokens = esprima.parseScript(code, { range: true, loc: true });
  const variables = getVariables(tokens);

  return { tokens, variables };
}

const parseBinaryExpressionValues = (expression, variables) => {
  const left =
    expression.left.type === 'Literal'
      ? expression.left.value
      : (expression.left.type === 'Identifier'
        ? variables[expression.left.name]
        : parseBinaryExpressionValues(expression.left, variables));

  const right =
    expression.right.type === 'Literal'
      ? expression.right.value
      : (expression.right.type === 'Identifier'
        ? variables[expression.right.name]
        : parseBinaryExpressionValues(expression.right, variables));

  return BINARY_OPS[expression.operator](left, right);
}

const isRequire = (node, variables) => {
  return node.type === 'CallExpression'
    && (node.callee.name === 'require'
      || (node.callee.type === 'MemberExpression'
        && (node.callee.computed === false
          && node.callee.property.type === 'Identifier'
          ? variables[node.callee.property.name] === 'require'
          : node.callee.property.type.value)
        || node.callee.computed === true && variables[node.callee.property.value] === 'require')
      || node.callee.type === 'Identifier' && variables[node.callee.name] === 'require');
}

const isEval = (node) => {
  return node.type === 'CallExpression' && node.callee.name === 'eval'
}

const isWhile = (node) => {
  return node.type === 'WhileStatement';
}

const isFor = (node) => {
  return node.type === 'ForStatement';
}

const hasLiteralArgument = (node) => {
  return node.arguments[0].type === 'Literal';
}

const hasIdentifierArgument = (node) => {
  return node.arguments[0].type === 'Identifier';
}

const hasBinaryExpressionArgument = (node) => {
  return node.arguments[0].type === 'BinaryExpression';
}

const hasCallExpressionArgument = (node) => {
  return node.arguments[0].type === 'CallExpression';
}

const whileHasLiteralArgument = (node) => {
  return node.test.type === 'Literal';
}

const whileHasIdentifierArgument = (node) => {
  return node.test.type === 'Identifier';
}

const whileHasBinaryExpressionArgument = (node) => {
  return node.test.type === 'BinaryExpression';
}

const whileHasCallExpressionArgument = (node) => {
  return node.test.type === 'CallExpression';
}

const forHasNoInitTestUpdate = (node) => {
  return node.init === null && node.test === null && node.update === null;
}

const forHasLiteralTest = (node) => {
  return node.test.type === 'Literal';
}

const forHasIdentifierTest = (node) => {
  return node.test.type === 'Identifier';
}

const forHasBinaryExpressionTest = (node) => {
  return node.test.type === 'BinaryExpression';
}

const walkRequires = (tokens, variables = {}, callback) => {
  walk(tokens, node => {
    if (isRequire(node, variables) && hasLiteralArgument(node)) {
      callback(node.arguments[0].value);
    } else if (isRequire(node, variables) && hasIdentifierArgument(node)) {
      callback(variables[node.arguments[0].name]);
    } else if (isRequire(node, variables) && hasBinaryExpressionArgument(node)) {
      const expression = node.arguments[0];
      const result = parseBinaryExpressionValues(expression, variables);
      callback(result);
    } else if (isRequire(node, variables) && hasCallExpressionArgument(node)) {
      // if the argument for require is a CallExpression, e.g require(fn())
      // just hardcode a forbidden module
      // TODO: parse the code inside the CallExpression
      callback('unknown');
    }
  });

  return tokens;
}

const walkEvals = (tokens, variables = {}, callback) => {
  walk(tokens, node => {
    if (isEval(node) && hasLiteralArgument(node)) {
      callback(node.arguments[0].value);
    } else if (isEval(node) && hasIdentifierArgument(node)) {
      callback(variables[node.arguments[0].name]);
    } else if (isEval(node) && hasBinaryExpressionArgument(node)) {
      const expression = node.arguments[0];
      const result = parseBinaryExpressionValues(expression, variables);
      callback(result);
    } else if (isEval(node) && hasCallExpressionArgument(node)) {
      // if the argument for eval is a CallExpression, e.g eval(fn())
      // just hardcode a forbidden module
      // TODO: parse the code inside the CallExpression
      callback('require("unknown")');
    }
  });
}

const walkWhiles = (tokens, variables, callback) => {
  walk(tokens, node => {
    if (isWhile(node) && whileHasLiteralArgument(node)) {
      callback(!!node.test.value);
    } else if (isWhile(node) && whileHasIdentifierArgument(node)) {
      callback(variables[node.test.name]);
    } else if (isWhile(node) && whileHasBinaryExpressionArgument(node)) {
      const expression = node.test;
      const result = parseBinaryExpressionValues(expression, variables);
      callback(!!result);
    } else if (isWhile(node) && whileHasCallExpressionArgument(node)) {
      // if the argument for while is a CallExpression, e.g while(fn())
      // just hardcode a inifinite loop
      callback(true);
    }
  });
}

const walkFors = (tokens, variables, callback) => {
  walk(tokens, node => {
    if (isFor(node) && forHasNoInitTestUpdate(node)) {
      callback(true);
    } else if (isFor(node) && forHasLiteralTest(node)) {
      callback(!!node.test.value);
    } else if (isFor(node) && forHasIdentifierTest(node)) {
      callback(variables[node.test.name]);
    } else if (isFor(node) && forHasBinaryExpressionTest(node)) {
      const expression = node.test;
      const result = parseBinaryExpressionValues(expression, variables);
      callback(!!result);
    }
  });
}

const hasForbiddenRequires = ({ tokens, variables }) => {
  let found = false;

  walkRequires(tokens, variables, module => {
    found = isForbidden(module);
  });

  walkEvals(tokens, variables, code => {
    const { tokens, variables } = parseExternalCode(code);
    found = hasForbiddenRequires({ tokens, variables });
  });

  return found;
}

const hasInfiniteLoops = ({ tokens, variables }) => {
  let found = false;

  walkWhiles(tokens, variables, condition => {
    found = condition;
  });

  walkFors(tokens, variables, condition => {
    found = condition;
  });

  walkEvals(tokens, variables, code => {
    const { tokens, variables } = parseExternalCode(code);
    found = hasInfiniteLoops({ tokens, variables });
  });

  return found;
}

const getForbiddenRequires = ({ tokens, variables }) => {
  const forbiddenRequires = [];

  walkRequires(tokens, variables, module => {
    if (isForbidden(module)) forbiddenRequires.push(module);
  });

  return forbiddenRequires;
}

const runExternalCode = (code) => {
  const { tokens, variables } = parseExternalCode(code);

  const hasForbidden = hasForbiddenRequires({ tokens, variables });
  const hasInfinite = hasInfiniteLoops({ tokens, variables });

  if (hasInfinite) {
    throw new InfiniteLoopError();
  }

  if (hasForbidden) {
    const forbiddenRequires = getForbiddenRequires({ tokens, variables });
    throw new ForbiddenModuleError(forbiddenRequires);
  }

  let env = process.env;
  if (Object.keys(process.env).length > 0) {
    process.env = {};
  }

  const result = eval(code);
  process.env = env;

  return result;
}

module.exports = {
  parseExternalCode,
  hasForbiddenRequires,
  hasInfiniteLoops,
  run: runExternalCode
}
