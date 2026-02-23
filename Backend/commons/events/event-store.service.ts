@Injectable()
export class EventStoreService {
  constructor(
    @InjectRepository(EventEntity)
    private repo: Repository<EventEntity>,
  ) {}

  async save(event: BaseEvent) {
    await this.repo.save({
      id: event.id,
      type: event.type,
      version: event.version,
      payload: event.data,
    });
  }
}