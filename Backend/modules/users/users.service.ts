@Injectable()
export class UsersService {
  constructor(private readonly kafkaService: KafkaService) {}

  async createUser(dto: CreateUserDto) {
    // Save user in DB first

    const event = new UserCreatedEvent({
      userId: 'generated-id',
      email: dto.email,
    });

    await this.kafkaService.emit('user.events', event);

    return { message: 'User created and event published' };
  }
}