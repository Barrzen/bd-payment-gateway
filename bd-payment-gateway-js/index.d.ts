export type JsonInput = string | Record<string, unknown>;

export interface EnvironmentConfig {
  mode: 'sandbox' | 'production' | 'live' | 'custom';
  custom_base_url?: string;
}

export interface InitiatePaymentResponse {
  redirect_url: string;
  provider_reference: string;
  raw: unknown;
  request_id?: string;
}

export interface VerifyPaymentResponse {
  status: string;
  provider_reference: string;
  amount?: string;
  currency?: string;
  raw: unknown;
  request_id?: string;
}

export interface RefundResponse {
  status: string;
  provider_reference: string;
  raw: unknown;
  request_id?: string;
}

export interface ShurjopayConfig {
  username: string;
  password: string;
  prefix: string;
  environment: EnvironmentConfig;
}

export interface ShurjopayInitiateRequest {
  amount: string;
  order_id: string;
  currency: string;
  return_url: string;
  cancel_url: string;
  client_ip: string;
  customer_name: string;
  customer_phone: string;
  customer_email: string;
  customer_address: string;
  customer_city: string;
  customer_state: string;
  customer_postcode: string;
  customer_country: string;
  value1?: string;
  value2?: string;
  value3?: string;
  value4?: string;
  discount_amount?: string;
  discount_percent?: string;
  correlation_id?: string;
}

export interface ShurjopayVerifyRequest {
  sp_order_id: string;
  correlation_id?: string;
}

export interface PortwalletConfig {
  app_key: string;
  app_secret: string;
  environment: EnvironmentConfig;
}

export interface PortwalletCustomer {
  name: string;
  email: string;
  phone: string;
  address?: string;
  city?: string;
  zip_code?: string;
  country?: string;
}

export interface PortwalletInitiateRequest {
  order: string;
  amount: string;
  currency: string;
  redirect_url: string;
  ipn_url: string;
  reference?: string;
  customer: PortwalletCustomer;
  correlation_id?: string;
}

export interface PortwalletVerifyRequest {
  invoice_id: string;
  correlation_id?: string;
}

export interface PortwalletRefundRequest {
  invoice_id: string;
  amount: string;
  reason?: string;
  correlation_id?: string;
}

export interface AamarpayConfig {
  store_id: string;
  signature_key: string;
  environment: EnvironmentConfig;
}

export interface AamarpayInitiateRequest {
  tran_id: string;
  amount: string;
  currency: string;
  success_url: string;
  fail_url: string;
  cancel_url: string;
  desc?: string;
  cus_name: string;
  cus_email: string;
  cus_add1: string;
  cus_add2?: string;
  cus_city: string;
  cus_state?: string;
  cus_postcode?: string;
  cus_country: string;
  cus_phone: string;
  opt_a?: string;
  opt_b?: string;
  opt_c?: string;
  opt_d?: string;
  signature_key?: string;
}

export interface AamarpayVerifyRequest {
  request_id: string;
}

export interface SslcommerzConfig {
  store_id: string;
  store_passwd: string;
  environment: EnvironmentConfig;
}

export interface SslcommerzInitiateRequest {
  total_amount: string;
  currency: string;
  tran_id: string;
  success_url: string;
  fail_url: string;
  cancel_url: string;
  ipn_url?: string;
  shipping_method?: string;
  product_name: string;
  product_category: string;
  product_profile: string;
  cus_name: string;
  cus_email: string;
  cus_add1: string;
  cus_city: string;
  cus_country: string;
  cus_phone: string;
  value_a?: string;
  value_b?: string;
  value_c?: string;
  value_d?: string;
}

export type SslcommerzVerifyRequest =
  | { reference: { ValId: string } }
  | { reference: { SessionKey: string } }
  | { reference: { TranId: string } };

export type SslcommerzRefundRequest =
  | {
      Initiate: {
        bank_tran_id: string;
        refund_amount: string;
        refund_remarks: string;
      };
    }
  | { Query: { refund_ref_id: string } };

export class ShurjopayClient {
  initiatePayment(request: ShurjopayInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verifyPayment(request: ShurjopayVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
  initiate_payment(request: ShurjopayInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verify_payment(request: ShurjopayVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
}

export class PortwalletClient {
  initiatePayment(request: PortwalletInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verifyPayment(request: PortwalletVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
  refund(request: PortwalletRefundRequest | JsonInput): Promise<RefundResponse>;
  initiate_payment(request: PortwalletInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verify_payment(request: PortwalletVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
  refund_payment(request: PortwalletRefundRequest | JsonInput): Promise<RefundResponse>;
}

export class AamarpayClient {
  initiatePayment(request: AamarpayInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verifyPayment(request: AamarpayVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
  initiate_payment(request: AamarpayInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verify_payment(request: AamarpayVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
}

export class SslcommerzClient {
  initiatePayment(request: SslcommerzInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verifyPayment(request: SslcommerzVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
  refund(request: SslcommerzRefundRequest | JsonInput): Promise<RefundResponse>;
  initiate_payment(request: SslcommerzInitiateRequest | JsonInput): Promise<InitiatePaymentResponse>;
  verify_payment(request: SslcommerzVerifyRequest | JsonInput): Promise<VerifyPaymentResponse>;
  refund_payment(request: SslcommerzRefundRequest | JsonInput): Promise<RefundResponse>;
}

export function createShurjopayClient(config: ShurjopayConfig | JsonInput): ShurjopayClient;
export function createPortwalletClient(config: PortwalletConfig | JsonInput): PortwalletClient;
export function createAamarpayClient(config: AamarpayConfig | JsonInput): AamarpayClient;
export function createSslcommerzClient(config: SslcommerzConfig | JsonInput): SslcommerzClient;

// Backward-compatible snake_case aliases
export const create_shurjopay_client: typeof createShurjopayClient;
export const create_portwallet_client: typeof createPortwalletClient;
export const create_aamarpay_client: typeof createAamarpayClient;
export const create_sslcommerz_client: typeof createSslcommerzClient;
