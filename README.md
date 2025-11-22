# Online Boutique (Amimono version)

This is an Amimono version of GCP's [Online Boutique][online-boutique]
microservices demo.

[online-boutique]: https://github.com/GoogleCloudPlatform/microservices-demo

This app consists of the following Amimono components:

* (Axum) **frontend** &mdash; A web server.
* (RPC) **adservice** &mdash; Generates ads based on context keywords.
* (RPC) **cartservice** &mdash; Manage cart storage.
* (RPC) **checkoutservice** &mdash; Coordinates the checkout process.
* (RPC) **currencyservice** &mdash; Provides currency conversion.
* (RPC) **emailservice** &mdash; Sends order confirmation emails. (Does not actually do this.)
* (RPC) **paymentservice** &mdash; Process payment information. (Does not actually do this.)
* (RPC) **productcatalogservice** &mdash; Search products and retrieve product details.
* (RPC) **recommendationservice** &mdash; Get product recommendations.
* (RPC) **shippingservice** &mdash; Quote shipping costs and ship orders. (Does not actually do this.)

The frontend is an Axum component that serves static content (in `static/`) as
well as dynamically-generated HTML from compiled-in templates (in
`src/frontend/templates/`). The backend services are written with Amimono RPC.

## Running locally

* Run `cargo run -- --local`

* Open http://localhost:8123 in your browser

## Deploying to minikube

This demo contains config files (`Dockerfile` and `amimono.toml`) for building
the application and deploying it to minikube.

* Set up minikube and other tools. The following steps only need to be performed once:

  * Install [minikube][minikube] and [kubectl][kubectl]

  * Start minikube with `minikube start`

  * Bind the "view" role to the default service account:

    ```
    kubectl create rolebinding default-view \
      --clusterrole=view \
      --serviceaccount=default:default \
      --namespace=default
      ```

  * Install the Amimono CLI:

    ```
    cargo install --git https://github.com/aji/amimono.git amimono-cli
    ```

* Build and deploy the app:

  * Build: `minikube image build . -t amimono-demo-boutique`

  * Deploy: `ammn deploy minikube`

* You can access the app by running `minikube service frontend --url` and
  opening the provided URL in your browser.

[minikube]: https://minikube.sigs.k8s.io/docs/start/
[kubectl]: https://kubernetes.io/docs/tasks/tools/

The current revision of the app creates one Deployment per component, which is
the configuration described by `configure_strict_microservices` in
`src/main.rs`. Additionally there is a `configure_strict_monolith` configuration
which creates only one Deployment for the whole app. In both cases, one Service
is created per component that requests an HTTP binding, which in this demo is
all of them.