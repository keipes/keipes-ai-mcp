import { calculatorToolCallback } from "../src/tools/calculator.js";

async function testCalculator() {
  console.log("Testing calculator tool...");

  const addResult = await calculatorToolCallback({
    operation: "add",
    a: 5,
    b: 3,
  });

  console.log("5 + 3 =", addResult.content[0].text);

  const multiplyResult = await calculatorToolCallback({
    operation: "multiply",
    a: 4,
    b: 7,
  });

  console.log("4 * 7 =", multiplyResult.content[0].text);
}

testCalculator().catch(console.error);
