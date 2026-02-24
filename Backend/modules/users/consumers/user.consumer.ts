@Injectable()
export class UserConsumer implements OnModuleInit {
  constructor(private readonly kafkaService: KafkaService) {}

  async onModuleInit() {
    await this.kafkaService.subscribe('user.events', async (event) => {
      if (event.type === 'user.created') {
        console.log('User created event received:', event);
        // Perform async action (send email, analytics, etc.)
      }
    });
  }
}