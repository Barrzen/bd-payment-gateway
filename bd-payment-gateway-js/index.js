'use strict';

const path = require('node:path');

function loadNative() {
  const candidates = [
    './bd-payment-gateway-js.node',
    './bd_payment_gateway_js.node',
    './index.node',
    `./${process.platform}-${process.arch}/bd-payment-gateway-js.node`,
  ];

  let lastError;
  for (const candidate of candidates) {
    try {
      return require(path.join(__dirname, candidate));
    } catch (error) {
      lastError = error;
    }
  }

  throw new Error(
    `Unable to load native module for bd-payment-gateway-js. Tried: ${candidates.join(', ')}. Last error: ${lastError}`
  );
}

const native = loadNative();

function normalizeInput(input) {
  return typeof input === 'string' ? input : JSON.stringify(input);
}

function normalizeResponse(response) {
  if (!response || typeof response !== 'object') {
    return response;
  }

  if (typeof response.raw === 'string') {
    try {
      return { ...response, raw: JSON.parse(response.raw) };
    } catch (_) {
      return response;
    }
  }

  return response;
}

function requireNativeFunction(functionName, featureName) {
  const fn = native[functionName];
  if (typeof fn === 'function') {
    return fn;
  }
  throw new Error(
    `${functionName} is unavailable in this native build. Rebuild bd-payment-gateway-js with feature '${featureName}'.`
  );
}

class ShurjopayClient {
  constructor(inner) {
    this.inner = inner;
  }

  async initiatePayment(request) {
    return normalizeResponse(await this.inner.initiate_payment(normalizeInput(request)));
  }

  async verifyPayment(request) {
    return normalizeResponse(await this.inner.verify_payment(normalizeInput(request)));
  }

  async initiate_payment(request) {
    return this.initiatePayment(request);
  }

  async verify_payment(request) {
    return this.verifyPayment(request);
  }
}

class PortwalletClient {
  constructor(inner) {
    this.inner = inner;
  }

  async initiatePayment(request) {
    return normalizeResponse(await this.inner.initiate_payment(normalizeInput(request)));
  }

  async verifyPayment(request) {
    return normalizeResponse(await this.inner.verify_payment(normalizeInput(request)));
  }

  async refund(request) {
    return normalizeResponse(await this.inner.refund(normalizeInput(request)));
  }

  async initiate_payment(request) {
    return this.initiatePayment(request);
  }

  async verify_payment(request) {
    return this.verifyPayment(request);
  }

  async refund_payment(request) {
    return this.refund(request);
  }
}

class AamarpayClient {
  constructor(inner) {
    this.inner = inner;
  }

  async initiatePayment(request) {
    return normalizeResponse(await this.inner.initiate_payment(normalizeInput(request)));
  }

  async verifyPayment(request) {
    return normalizeResponse(await this.inner.verify_payment(normalizeInput(request)));
  }

  async initiate_payment(request) {
    return this.initiatePayment(request);
  }

  async verify_payment(request) {
    return this.verifyPayment(request);
  }
}

class SslcommerzClient {
  constructor(inner) {
    this.inner = inner;
  }

  async initiatePayment(request) {
    return normalizeResponse(await this.inner.initiate_payment(normalizeInput(request)));
  }

  async verifyPayment(request) {
    return normalizeResponse(await this.inner.verify_payment(normalizeInput(request)));
  }

  async refund(request) {
    return normalizeResponse(await this.inner.refund(normalizeInput(request)));
  }

  async initiate_payment(request) {
    return this.initiatePayment(request);
  }

  async verify_payment(request) {
    return this.verifyPayment(request);
  }

  async refund_payment(request) {
    return this.refund(request);
  }
}

function createShurjopayClient(config) {
  const factory = requireNativeFunction('create_shurjopay_client', 'shurjopay');
  return new ShurjopayClient(factory(normalizeInput(config)));
}

function createPortwalletClient(config) {
  const factory = requireNativeFunction('create_portwallet_client', 'portwallet');
  return new PortwalletClient(factory(normalizeInput(config)));
}

function createAamarpayClient(config) {
  const factory = requireNativeFunction('create_aamarpay_client', 'aamarpay');
  return new AamarpayClient(factory(normalizeInput(config)));
}

function createSslcommerzClient(config) {
  const factory = requireNativeFunction('create_sslcommerz_client', 'sslcommerz');
  return new SslcommerzClient(factory(normalizeInput(config)));
}

module.exports = {
  createShurjopayClient,
  createPortwalletClient,
  createAamarpayClient,
  createSslcommerzClient,
  ShurjopayClient,
  PortwalletClient,
  AamarpayClient,
  SslcommerzClient,
  // Backward-compatible snake_case aliases
  create_shurjopay_client: createShurjopayClient,
  create_portwallet_client: createPortwalletClient,
  create_aamarpay_client: createAamarpayClient,
  create_sslcommerz_client: createSslcommerzClient,
};
