export class ProtocolHandler {
  validateMessage(message: any): boolean {
    return message && typeof message === "object";
  }

  processMessage(message: any) {
    return message;
  }
}
