import { Injectable } from '@nestjs/common';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';

export interface ApiEndpoint {
  path: string;
  method: string;
  description: string;
  parameters: ApiParameter[];
  requestBody?: any;
  responses: ApiResponse[];
  examples: any[];
}

export interface ApiParameter {
  name: string;
  type: string;
  required: boolean;
  description: string;
  example?: any;
}

export interface ApiDocumentation {
  title: string;
  version: string;
  description: string;
  baseUrl: string;
  endpoints: ApiEndpoint[];
  schemas: any[];
  lastUpdated: Date;
}

@Injectable()
export class ApiDocumentationService {
  private documentation: ApiDocumentation;

  constructor() {
    this.initializeDocumentation();
  }

  private initializeDocumentation(): void {
    this.documentation = {
      title: 'Stellara API',
      version: '2.0.0',
      description: 'Comprehensive API for blockchain transactions, smart contracts, and DeFi operations',
      baseUrl: process.env.API_BASE_URL || 'https://api.stellara.io/v2',
      endpoints: [],
      schemas: [],
      lastUpdated: new Date(),
    };
  }

  async generateDocumentation(): Promise<ApiDocumentation> {
    // Scan all controllers and extract API information
    const endpoints = await this.scanEndpoints();
    
    this.documentation.endpoints = endpoints;
    this.documentation.lastUpdated = new Date();
    
    return this.documentation;
  }

  async generateInteractiveDocs(): Promise<any> {
    const docs = await this.generateDocumentation();
    
    return {
      openapi: '3.0.0',
      info: {
        title: docs.title,
        version: docs.version,
        description: docs.description,
      },
      servers: [
        {
          url: docs.baseUrl,
          description: 'Production server',
        },
      ],
      paths: this.convertToOpenApiPaths(docs.endpoints),
      components: {
        schemas: this.convertToOpenApiSchemas(docs.schemas),
      },
    };
  }

  async updateDocumentation(): Promise<void> {
    // Automatically update documentation when code changes
    const newEndpoints = await this.scanEndpoints();
    
    // Compare with existing endpoints and update if needed
    const hasChanges = this.detectChanges(newEndpoints);
    
    if (hasChanges) {
      this.documentation.endpoints = newEndpoints;
      this.documentation.lastUpdated = new Date();
      
      // Notify subscribers of documentation updates
      await this.notifyDocumentationUpdate();
    }
  }

  private async scanEndpoints(): Promise<ApiEndpoint[]> {
    // In a real implementation, this would use reflection or decorators
    // to automatically scan all controllers and extract endpoint information
    
    return [
      {
        path: '/transactions',
        method: 'POST',
        description: 'Create a new blockchain transaction',
        parameters: [
          {
            name: 'amount',
            type: 'number',
            required: true,
            description: 'Transaction amount in smallest currency unit',
            example: 1000000,
          },
          {
            name: 'recipient',
            type: 'string',
            required: true,
            description: 'Recipient wallet address',
            example: 'GD5...',
          },
        ],
        responses: [
          {
            status: 201,
            description: 'Transaction created successfully',
            schema: { type: 'object', properties: { transactionId: { type: 'string' } } },
          },
        ],
        examples: [
          {
            request: { amount: 1000000, recipient: 'GD5...' },
            response: { transactionId: 'tx_123456789' },
          },
        ],
      },
      {
        path: '/transactions/{id}',
        method: 'GET',
        description: 'Get transaction details',
        parameters: [
          {
            name: 'id',
            type: 'string',
            required: true,
            description: 'Transaction ID',
            example: 'tx_123456789',
          },
        ],
        responses: [
          {
            status: 200,
            description: 'Transaction details',
            schema: {
              type: 'object',
              properties: {
                id: { type: 'string' },
                amount: { type: 'number' },
                status: { type: 'string' },
                timestamp: { type: 'string', format: 'date-time' },
              },
            },
          },
        ],
        examples: [],
      },
    ];
  }

  private detectChanges(newEndpoints: ApiEndpoint[]): boolean {
    // Compare new endpoints with existing ones
    if (this.documentation.endpoints.length !== newEndpoints.length) {
      return true;
    }

    return newEndpoints.some((newEndpoint, index) => {
      const existingEndpoint = this.documentation.endpoints[index];
      return !this.endpointsEqual(newEndpoint, existingEndpoint);
    });
  }

  private endpointsEqual(endpoint1: ApiEndpoint, endpoint2: ApiEndpoint): boolean {
    return (
      endpoint1.path === endpoint2.path &&
      endpoint1.method === endpoint2.method &&
      endpoint1.description === endpoint2.description
    );
  }

  private convertToOpenApiPaths(endpoints: ApiEndpoint[]): any {
    const paths: any = {};
    
    endpoints.forEach(endpoint => {
      if (!paths[endpoint.path]) {
        paths[endpoint.path] = {};
      }
      
      const operation = {
        summary: endpoint.description,
        description: endpoint.description,
        parameters: endpoint.parameters.map(param => ({
          name: param.name,
          in: 'query',
          required: param.required,
          schema: { type: param.type },
          description: param.description,
        })),
        responses: this.convertToOpenApiResponses(endpoint.responses),
        examples: endpoint.examples,
      };
      
      paths[endpoint.path][endpoint.method.toLowerCase()] = operation;
    });
    
    return paths;
  }

  private convertToOpenApiResponses(responses: any[]): any {
    const openApiResponses: any = {};
    
    responses.forEach(response => {
      openApiResponses[response.status] = {
        description: response.description,
        content: {
          'application/json': {
            schema: response.schema,
          },
        },
      };
    });
    
    return openApiResponses;
  }

  private convertToOpenApiSchemas(schemas: any[]): any {
    const openApiSchemas: any = {};
    
    schemas.forEach(schema => {
      openApiSchemas[schema.name] = schema;
    });
    
    return openApiSchemas;
  }

  private async notifyDocumentationUpdate(): Promise<void> {
    // Send webhook notifications to subscribed developers
    // Update documentation cache
    // Trigger re-generation of SDKs
    console.log('Documentation updated - notifying subscribers');
  }

  async getDocumentation(): Promise<ApiDocumentation> {
    return this.documentation;
  }

  async getInteractiveDocumentation(): Promise<any> {
    return this.generateInteractiveDocs();
  }

  async validateEndpoint(endpointPath: string, method: string): Promise<boolean> {
    const endpoint = this.documentation.endpoints.find(
      ep => ep.path === endpointPath && ep.method.toLowerCase() === method.toLowerCase()
    );
    
    return !!endpoint;
  }

  async getEndpointExamples(endpointPath: string, method: string): Promise<any[]> {
    const endpoint = this.documentation.endpoints.find(
      ep => ep.path === endpointPath && ep.method.toLowerCase() === method.toLowerCase()
    );
    
    return endpoint?.examples || [];
  }
}
