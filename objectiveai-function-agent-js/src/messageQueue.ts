export class MessageQueue {
  private messages: string[] = [];
  private waiter: (() => void) | null = null;

  push(message: string): void {
    this.messages.push(message);
    if (this.waiter) {
      const resolve = this.waiter;
      this.waiter = null;
      resolve();
    }
  }

  drain(): string[] {
    return this.messages.splice(0);
  }

  get length(): number {
    return this.messages.length;
  }

  waitForMessage(): Promise<void> {
    if (this.messages.length > 0) return Promise.resolve();
    return new Promise<void>((resolve) => {
      this.waiter = resolve;
    });
  }
}
