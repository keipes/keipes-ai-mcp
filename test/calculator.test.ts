import { calculatorTool } from "../src/tools/calculator.js";

async function testCalculator() {
  console.log("Testing calculator tool...");

  const addResult = await calculatorTool.handler({
    operation: "add",
    a: 5,
    b: 3,
  });

  console.log("5 + 3 =", addResult.result);

  const multiplyResult = await calculatorTool.handler({
    operation: "multiply",
    a: 4,
    b: 7,
  });

  console.log("4 * 7 =", multiplyResult.result);
}

testCalculator().catch(console.error);
