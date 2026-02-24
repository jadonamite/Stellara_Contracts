import { Injectable } from '@nestjs/common';

export interface SdkConfig {
  language: string;
  version: string;
  baseUrl: string;
  authentication: {
    type: 'api-key' | 'oauth2' | 'jwt';
    headerName?: string;
  };
  endpoints: SdkEndpoint[];
}

export interface SdkEndpoint {
  name: string;
  path: string;
  method: string;
  parameters: SdkParameter[];
  returnType: string;
}

export interface SdkParameter {
  name: string;
  type: string;
  required: boolean;
  description: string;
}

export interface GeneratedSdk {
  language: string;
  version: string;
  content: string;
  fileName: string;
  dependencies: string[];
  examples: string[];
}

@Injectable()
export class SdkGenerationService {
  private supportedLanguages = ['javascript', 'python', 'typescript', 'java', 'go', 'rust'];
  private sdkTemplates: Map<string, string>;

  constructor() {
    this.initializeTemplates();
  }

  private initializeTemplates(): void {
    this.sdkTemplates = new Map([
      ['javascript', this.getJavaScriptTemplate()],
      ['python', this.getPythonTemplate()],
      ['typescript', this.getTypeScriptTemplate()],
      ['java', this.getJavaTemplate()],
      ['go', this.getGoTemplate()],
      ['rust', this.getRustTemplate()],
    ]);
  }

  async generateSdk(config: SdkConfig): Promise<GeneratedSdk> {
    const template = this.sdkTemplates.get(config.language);
    
    if (!template) {
      throw new Error(`Unsupported language: ${config.language}`);
    }

    const content = this.populateTemplate(template, config);
    const fileName = this.getFileName(config.language);
    const dependencies = this.getDependencies(config.language);
    const examples = this.generateExamples(config);

    return {
      language: config.language,
      version: config.version,
      content,
      fileName,
      dependencies,
      examples,
    };
  }

  async generateAllSdks(config: Omit<SdkConfig, 'language'>): Promise<GeneratedSdk[]> {
    const sdks: GeneratedSdk[] = [];

    for (const language of this.supportedLanguages) {
      try {
        const sdk = await this.generateSdk({ ...config, language });
        sdks.push(sdk);
      } catch (error) {
        console.error(`Failed to generate SDK for ${language}:`, error);
      }
    }

    return sdks;
  }

  private populateTemplate(template: string, config: SdkConfig): string {
    return template
      .replace('{{VERSION}}', config.version)
      .replace('{{BASE_URL}}', config.baseUrl)
      .replace('{{AUTH_TYPE}}', config.authentication.type)
      .replace('{{AUTH_HEADER}}', config.authentication.headerName || 'Authorization')
      .replace('{{ENDPOINTS}}', this.generateEndpointMethods(config.endpoints));
  }

  private generateEndpointMethods(endpoints: SdkEndpoint[]): string {
    return endpoints.map(endpoint => {
      switch (endpoint.language) {
        case 'javascript':
        return this.generateJavaScriptMethod(endpoint);
        case 'python':
          return this.generatePythonMethod(endpoint);
        case 'typescript':
          return this.generateTypeScriptMethod(endpoint);
        case 'java':
          return this.generateJavaMethod(endpoint);
        case 'go':
          return this.generateGoMethod(endpoint);
        case 'rust':
          return this.generateRustMethod(endpoint);
        default:
          return '';
      }
    }).join('\n\n');
  }

  private getJavaScriptTemplate(): string {
    return `
/**
 * Stellara SDK v{{VERSION}}
 * JavaScript client for Stellara API
 */

class StellaraClient {
  constructor(apiKey, baseUrl = '{{BASE_URL}}') {
    this.apiKey = apiKey;
    this.baseUrl = baseUrl;
    this.headers = {
      '{{AUTH_HEADER}}': '{{AUTH_TYPE}} ' + apiKey,
      'Content-Type': 'application/json'
    };
  }

  {{ENDPOINTS}}

  async request(method, path, data = null) {
    const config = {
      method,
      url: \`\${this.baseUrl}\${path}\`,
      headers: this.headers,
    };

    if (data) {
      config.body = JSON.stringify(data);
    }

    const response = await fetch(path, config);
    return response.json();
  }
}

module.exports = StellaraClient;
`;
  }

  private getPythonTemplate(): string {
    return `
"""
Stellara SDK v{{VERSION}}
Python client for Stellara API
"""

import requests
import json

class StellaraClient:
    def __init__(self, api_key, base_url="{{BASE_URL}}"):
        self.api_key = api_key
        self.base_url = base_url
        self.headers = {
            "{{AUTH_HEADER}}": "{{AUTH_TYPE}} " + api_key,
            "Content-Type": "application/json"
        }

    {{ENDPOINTS}}

    def _request(self, method, path, data=None):
        url = f"{self.base_url}{path}"
        response = requests.request(method, url, headers=self.headers, json=data)
        return response.json()
`;
  }

  private getTypeScriptTemplate(): string {
    return `
/**
 * Stellara SDK v{{VERSION}}
 * TypeScript client for Stellara API
 */

interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

class StellaraClient {
  private apiKey: string;
  private baseUrl: string;
  private headers: Record<string, string>;

  constructor(apiKey: string, baseUrl: string = '{{BASE_URL}}') {
    this.apiKey = apiKey;
    this.baseUrl = baseUrl;
    this.headers = {
      '{{AUTH_HEADER}}': '{{AUTH_TYPE}} ' + apiKey,
      'Content-Type': 'application/json'
    };
  }

  {{ENDPOINTS}}

  private async request<T>(method: string, path: string, data?: any): Promise<ApiResponse<T>> {
    const config: RequestInit = {
      method,
      headers: this.headers,
    };

    if (data) {
      config.body = JSON.stringify(data);
    }

    const response = await fetch(\`\${this.baseUrl}\${path}\`, config);
    return response.json();
  }
}

export default StellaraClient;
`;
  }

  private getJavaTemplate(): string {
    return `
package com.stellara.sdk;

import java.net.http.*;
import java.net.URI;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Stellara SDK v{{VERSION}}
 * Java client for Stellara API
 */
public class StellaraClient {
    private final String apiKey;
    private final String baseUrl;
    private final HttpClient client;
    private final ObjectMapper mapper;

    public StellaraClient(String apiKey) {
        this(apiKey, "{{BASE_URL}}");
    }

    public StellaraClient(String apiKey, String baseUrl) {
        this.apiKey = apiKey;
        this.baseUrl = baseUrl;
        this.client = HttpClient.newHttpClient();
        this.mapper = new ObjectMapper();
    }

    {{ENDPOINTS}}

    private <T> T request(String method, String path, Object data) throws Exception {
        HttpRequest.Builder builder = HttpRequest.newBuilder()
            .uri(URI.create(baseUrl + path))
            .header("{{AUTH_HEADER}}", "{{AUTH_TYPE}} " + apiKey)
            .header("Content-Type", "application/json");

        if (data != null) {
            builder.POST(HttpRequest.BodyPublishers.ofString(mapper.writeValueAsString(data)));
        } else {
            builder.method(method, HttpRequest.BodyPublishers.noBody());
        }

        HttpResponse<String> response = client.send(builder.build(), HttpResponse.BodyHandlers.ofString());
        return mapper.readValue(response.body(), new TypeReference<T>() {});
    }
}
`;
  }

  private getGoTemplate(): string {
    return `
package stellara

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
    "io/ioutil"
)

// Stellara SDK v{{VERSION}}
// Go client for Stellara API

type StellaraClient struct {
    apiKey     string
    baseUrl    string
    httpClient  *http.Client
}

func NewStellaraClient(apiKey string) *StellaraClient {
    return NewStellaraClientWithBaseUrl(apiKey, "{{BASE_URL}}")
}

func NewStellaraClientWithBaseUrl(apiKey, baseUrl string) *StellaraClient {
    return &StellaraClient{
        apiKey:    apiKey,
        baseUrl:   baseUrl,
        httpClient: &http.Client{},
    }
}

{{ENDPOINTS}}

func (c *StellaraClient) request(method, path string, data interface{}) ([]byte, error) {
    url := c.baseUrl + path
    
    jsonData, err := json.Marshal(data)
    if err != nil {
        return nil, err
    }
    
    req, err := http.NewRequest(method, url, bytes.NewBuffer(jsonData))
    if err != nil {
        return nil, err
    }
    
    req.Header.Set("{{AUTH_HEADER}}", "{{AUTH_TYPE}} "+c.apiKey)
    req.Header.Set("Content-Type", "application/json")
    
    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()
    
    return ioutil.ReadAll(resp.Body)
}
`;
  }

  private getRustTemplate(): string {
    return `
//! Stellara SDK v{{VERSION}}
//! Rust client for Stellara API

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct StellaraClient {
    api_key: String,
    base_url: String,
    client: Client,
}

impl StellaraClient {
    pub fn new(api_key: &str) -> Self {
        Self::new_with_base_url(api_key, "{{BASE_URL}}")
    }

    pub fn new_with_base_url(api_key: &str, base_url: &str) -> Self {
        StellaraClient {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    {{ENDPOINTS}}

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        path: &str,
        data: Option<&impl Serialize>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        
        let request = self.client
            .request(method, &url)
            .header("{{AUTH_HEADER}}", &format!("{{AUTH_TYPE}} {}", self.api_key))
            .header("Content-Type", "application/json");

        let request = if let Some(data) = data {
            request.json(data)
        } else {
            request
        };

        let response = request.send().await?;
        let result: T = response.json().await?;
        Ok(result)
    }
}
`;
  }

  private getFileName(language: string): string {
    const extensions = {
      javascript: 'stellara-sdk.js',
      typescript: 'stellara-sdk.ts',
      python: 'stellara_sdk.py',
      java: 'StellaraClient.java',
      go: 'stellara.go',
      rust: 'stellara.rs',
    };

    return extensions[language] || 'stellara-sdk';
  }

  private getDependencies(language: string): string[] {
    const dependencies = {
      javascript: ['node-fetch'],
      typescript: ['node-fetch', '@types/node'],
      python: ['requests'],
      java: ['com.fasterxml.jackson.core:jackson-databind:2.13.0'],
      go: ['github.com/go-resty/resty/v2'],
      rust: ['reqwest', 'serde', 'serde_json'],
    };

    return dependencies[language] || [];
  }

  private generateExamples(config: SdkConfig): string[] {
    return config.endpoints.map(endpoint => {
      return `
// ${endpoint.name} example
${this.generateExampleForLanguage(config.language, endpoint)}
      `.trim();
    });
  }

  private generateExampleForLanguage(language: string, endpoint: SdkEndpoint): string {
    const examples = {
      javascript: `const result = await client.${endpoint.name}(${endpoint.parameters.map(p => `'${p.example || 'value'}'`).join(', ')});`,
      python: `result = client.${endpoint.name}(${endpoint.parameters.map(p => `'${p.example || 'value'}'`).join(', ')})`,
      typescript: `const result = await client.${endpoint.name}(${endpoint.parameters.map(p => `'${p.example || 'value'}'`).join(', ')});`,
      java: `var result = client.${endpoint.name}(${endpoint.parameters.map(p => `'${p.example || 'value'}'`).join(', ')});`,
      go: `result, err := client.${endpoint.name}(${endpoint.parameters.map(p => `'${p.example || 'value'}'`).join(', ')})`,
      rust: `let result = client.${endpoint.name}(${endpoint.parameters.map(p => `'${p.example || 'value'}'`).join(', ')}).await?;`,
    };

    return examples[language] || '';
  }

  async validateSdk(sdk: GeneratedSdk): Promise<{ valid: boolean; errors: string[] }> {
    const errors: string[] = [];

    // Check for syntax errors
    try {
      // Basic syntax validation would go here
      // This is a simplified check
      if (sdk.content.includes('undefined')) {
        errors.push('Potential undefined references found');
      }
    } catch (error) {
      errors.push(`Syntax error: ${error.message}`);
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  getSupportedLanguages(): string[] {
    return [...this.supportedLanguages];
  }
}
