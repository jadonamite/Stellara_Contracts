export interface BaseEvent<T = any> {
  id: string;               // UUID
  type: string;             // e.g. user.created
  version: number;          // schema version
  timestamp: string;        // ISO date
  data: T;                  // event payload
  metadata?: {
    correlationId?: string;
    source?: string;
  };
}