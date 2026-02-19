export interface InitiatePaymentResponse {
  redirect_url: string;
  provider_reference: string;
  raw: string;
  request_id?: string;
}

export interface VerifyPaymentResponse {
  status: string;
  provider_reference: string;
  amount?: string;
  currency?: string;
  raw: string;
  request_id?: string;
}

export interface RefundResponse {
  status: string;
  provider_reference: string;
  raw: string;
  request_id?: string;
}

export class ShurjopayClient {
  initiate_payment(requestJson: string): Promise<InitiatePaymentResponse>;
  verify_payment(requestJson: string): Promise<VerifyPaymentResponse>;
}

export class PortwalletClient {
  initiate_payment(requestJson: string): Promise<InitiatePaymentResponse>;
  verify_payment(requestJson: string): Promise<VerifyPaymentResponse>;
  refund(requestJson: string): Promise<RefundResponse>;
}

export class AamarpayClient {
  initiate_payment(requestJson: string): Promise<InitiatePaymentResponse>;
  verify_payment(requestJson: string): Promise<VerifyPaymentResponse>;
}

export class SslcommerzClient {
  initiate_payment(requestJson: string): Promise<InitiatePaymentResponse>;
  verify_payment(requestJson: string): Promise<VerifyPaymentResponse>;
  refund(requestJson: string): Promise<RefundResponse>;
}

export function create_shurjopay_client(configJson: string): ShurjopayClient;
export function create_portwallet_client(configJson: string): PortwalletClient;
export function create_aamarpay_client(configJson: string): AamarpayClient;
export function create_sslcommerz_client(configJson: string): SslcommerzClient;
